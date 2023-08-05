//! # Attestation Scheduler
//! Starts attributing to a price at the start of each slot.
//! Depending on how we set RUN_SLOT_LIMIT_SECS we never hit the case of trying to schedule a third
//! slot, while one is still running.

use std::sync::{Arc, Mutex};

use chrono::{Duration, Utc};
use eyre::{Context, Result};
use futures::StreamExt;
use lazy_static::lazy_static;
use tokio::time::{interval, timeout};
use tokio_stream::wrappers::IntervalStream;

use crate::{
    message_broadcaster::MessageBroadcaster,
    message_generator::MessageGenerator,
    price_provider::{PriceProvider, PRECISION_FACTOR},
    slot::Slot,
};

// We set a limit,, although the fact slots appear every 12s, and attestations can take at most
// 24s to process, means we run at most 2 attestations at any time as long as timeouts are handled
// quickly.
const MAX_CONCURRENT_SLOTS: usize = 2;
const ATTESTATION_TIMEOUT: u64 = 24;
const SLOT_PERIOD_DURATION_SECS: u64 = 12;

lazy_static! {
    static ref SLOT_PERIOD: tokio::time::Duration =
        tokio::time::Duration::from_secs(SLOT_PERIOD_DURATION_SECS);
    static ref ATTESTATION_TIMEOUT_DURATION: tokio::time::Duration =
        tokio::time::Duration::from_secs(ATTESTATION_TIMEOUT);
    static ref DELAYED_START_LIMIT: Duration = Duration::milliseconds(1000);
}

/// Waits until the start of the next slot.
/// Much of our code depends on what the current slot is, and wants to answer as fast as possible,
/// therefore we sometimes want to align our code with the start of the slot.
async fn wait_until_next_slot() {
    let current_slot = Slot::now();
    let next_slot = current_slot.next();
    let next_slot_start = next_slot.to_date_time();
    // NOTE: doesn't account for leap seconds.
    let milliseconds_until_next_slot =
        next_slot_start.timestamp_millis() - Utc::now().timestamp_millis();

    tokio::time::sleep(tokio::time::Duration::from_millis(
        milliseconds_until_next_slot as u64,
    ))
    .await;
}

pub struct SystemClockAttestationScheduler<A: MessageBroadcaster, B: PriceProvider> {
    message_broadcaster: A,
    message_generator: MessageGenerator,
    price_provider: B,
    slots_to_run: Arc<Mutex<Option<u64>>>,
}

impl<A: MessageBroadcaster, B: PriceProvider> SystemClockAttestationScheduler<A, B> {
    pub fn new(
        message_broadcaster: A,
        message_generator: MessageGenerator,
        price_provider: B,
        slots_to_run: Option<u64>,
    ) -> Self {
        Self {
            message_broadcaster,
            message_generator,
            price_provider,
            slots_to_run: Arc::new(Mutex::new(slots_to_run)),
        }
    }

    async fn run_single_slot(&self, slot: Slot) -> Result<()> {
        log::info!("Running for slot: {}", slot);
        let start_time = chrono::Utc::now().timestamp();
        let price = self
            .price_provider
            .get_price()
            .await
            .wrap_err("Failed to get price data")?;
        log::info!(
            "Sucessfully obtained current Eth Price: {:?} for slot {} after {} seconds",
            price.value as f64 / PRECISION_FACTOR as f64,
            slot,
            chrono::Utc::now().timestamp() - start_time,
        );
        let oracle_message = self
            .message_generator
            .generate_oracle_message(price.clone(), slot)
            .wrap_err("Failed to generated signed price message")?;
        log::info!(
            "Sucessfully generated signed price message for slot {} after {} seconds",
            slot,
            chrono::Utc::now().timestamp() - start_time
        );
        self.message_broadcaster
            .broadcast(&oracle_message)
            .await
            .wrap_err("Failed to broadcast message")?;
        log::info!(
            "Sucessfully finished for slot {} after {} seconds",
            slot,
            chrono::Utc::now().timestamp() - start_time
        );
        Ok(())
    }

    pub async fn run(&self) {
        log::debug!("Waiting until next slot to start interval stream");
        wait_until_next_slot().await;

        // This is probably broken for leap seconds. A slot is sometimes 11s, sometimes
        // 13s long. Assuming IntervalStream waits a number of real seconds, not unix timestamp
        // seconds, we'd become misaligned. For now, operators will simply have to restart after a
        // leap second.
        let interval_stream = IntervalStream::new(interval(*SLOT_PERIOD));

        let slots_to_run = self.slots_to_run.clone();

        interval_stream
            .take_while(move |_| {
                let mut slots_left_guard = slots_to_run.lock().unwrap();
                let should_continue = match *slots_left_guard {
                    Some(ref mut count) => {
                        let run_slot = *count > 0;
                        if run_slot {
                            *count -= 1;
                        }
                        run_slot
                    }
                    None => true,
                };
                if !should_continue {
                    log::info!("Max slots reached, stopping.");
                }
                futures::future::ready(should_continue)
            })
            .for_each_concurrent(MAX_CONCURRENT_SLOTS, |_| async {
                let slot = Slot::now();
                let now = Utc::now();

                // This means the previous two slots failed to complete within their 24 available
                // seconds. Because we want to start attesting as early as possible and use
                // resources sparingly, we use a limit.
                let millis_into_slot = now - slot.to_date_time();
                if millis_into_slot > *DELAYED_START_LIMIT {
                    log::warn!(
                        "Slot started more than 1000ms into the slot, skipping. Slot: {}, millis_into_slot: {}ms",
                        slot,
                        millis_into_slot.num_milliseconds()
                    );
                    return;
                }

                log::debug!(
                    "Attesting for slot with number: {}, slot start: {}, attestation started at: {}, delta: {}ms",
                    slot,
                    slot.to_date_time(),
                    now,
                    (now - slot.to_date_time()).num_milliseconds()
                );

                timeout(
                    *ATTESTATION_TIMEOUT_DURATION,
                    self.run_single_slot(slot),
                )
                    .await
                    .unwrap_or_else(|_| {
                        log::error!("Hit {}s timeout for slot: {}", ATTESTATION_TIMEOUT, slot);
                        Ok(())
                    })
                    .unwrap_or_else(|e| {
                        log::error!("Error when running for slot: {} - {:?}", slot, e);
                    });
            }).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        message_broadcaster::{json::JsonFileMessageBroadcaster, OracleMessage},
        price_provider::gofer::GoferPriceProvider,
        signature_provider::{private_key::PrivateKeySignatureProvider, SignatureProvider},
    };
    use std::fs;

    fn get_output_files() -> Vec<String> {
        let mut output_files = Vec::new();
        let paths = fs::read_dir("test_data/output").unwrap();
        for path in paths {
            let path = path.unwrap().path();
            let path = path.to_str().unwrap().to_string();
            output_files.push(path);
        }
        output_files
    }

    fn subtract_vecs(a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
        let mut c = a.clone();
        c.retain(|x| !b.contains(x));
        c
    }

    #[tokio::test]
    // Basic integration tests mocking out gofer with a static file
    async fn generates_oracle_message() {
        env_logger::init();

        // Create output directory if it doesn't exist
        fs::create_dir_all("./test_data/output").unwrap();

        let mut server = mockito::Server::new();

        let response_json = r#"{
                    "type":"aggregator",
                    "base":"ETH",
                    "quote":"USD",
                    "price":1953,
                    "bid":1953,
                    "ask":1952,
                    "vol24h":0,
                    "ts":"2023-07-04T15:55:48Z",
                    "prices":[]
                }"#;

        // Create a mock
        let mock = server
            .mock("POST", "/price")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json)
            .create();

        let url = format!("{}{}", server.url().as_str(), "/price");
        let price_provider = GoferPriceProvider::new(url.as_str());

        let signature_provider = PrivateKeySignatureProvider::random();
        let public_key = signature_provider.get_public_key().unwrap();
        let message_generator = MessageGenerator::new(Box::new(signature_provider));
        let output_files_before = get_output_files();
        let message_broadcaster =
            JsonFileMessageBroadcaster::new(Some("test_data/output".to_string())).unwrap();

        let attestation_scheduler = SystemClockAttestationScheduler::new(
            message_broadcaster,
            message_generator,
            price_provider,
            Some(1),
        );

        attestation_scheduler.run().await;

        let output_files_after = get_output_files();

        let new_output_files = subtract_vecs(&output_files_after, &output_files_before);
        assert_eq!(new_output_files.len(), 1);
        let new_output_file = fs::File::open(new_output_files.get(0).unwrap()).unwrap();
        let oracle_message: OracleMessage = serde_json::from_reader(new_output_file).unwrap();

        assert_eq!(oracle_message.validator_public_key, public_key);
        assert!(oracle_message.interval_inclusion_messages.len() > 100);

        mock.assert();
    }
}
