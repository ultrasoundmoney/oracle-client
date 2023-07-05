use std::sync::Arc;

use chrono::Utc;
use eyre::{Result, WrapErr};
use futures::StreamExt;
use tokio::{sync::mpsc, time::timeout};
use tokio_stream::wrappers::ReceiverStream;

mod message_broadcaster;
use message_broadcaster::{http::HttpMessageBroadcaster, MessageBroadcaster};
mod message_generator;
use message_generator::MessageGenerator;
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PriceProvider, PRECISION_FACTOR};
mod signature_provider;
use signature_provider::private_key::PrivateKeySignatureProvider;
mod slot_provider;
use slot_provider::{clock::SystemClockSlotProvider, slot::Slot, SlotProvider};

const MAX_CONCURRENT_SLOTS: usize = 8;
const RUN_SLOT_LIMIT_SECS: u64 = 24;

async fn run_single_slot(
    price_provider: Arc<impl PriceProvider>,
    message_generator: MessageGenerator,
    message_broadcaster: Arc<impl MessageBroadcaster>,
    slot: Slot,
) -> Result<()> {
    log::info!("Running for slot: {}", slot);
    let slot_number = slot;
    let start_time = chrono::Utc::now().timestamp();
    let price = price_provider
        .get_price()
        .wrap_err("Failed to get price data")?;
    log::info!(
        "Sucessfully obtained current Eth Price: {:?} for slot {} after {} seconds",
        price.value as f64 / PRECISION_FACTOR as f64,
        slot_number,
        chrono::Utc::now().timestamp() - start_time,
    );
    let oracle_message = &message_generator
        .generate_oracle_message(price.clone(), slot)
        .wrap_err("Failed to generated signed price message")?;
    log::info!(
        "Sucessfully generated signed price message for slot {} after {} seconds",
        slot_number,
        chrono::Utc::now().timestamp() - start_time
    );
    message_broadcaster
        .broadcast(oracle_message)
        .await
        .wrap_err("Failed to broadcast message")?;
    log::info!(
        "Sucessfully finished for slot {} after {} seconds",
        slot_number,
        chrono::Utc::now().timestamp() - start_time
    );
    Ok(())
}

async fn run_oracle_node(
    price_provider: Arc<impl PriceProvider>,
    message_generator: MessageGenerator,
    message_broadcaster: Arc<impl MessageBroadcaster>,
    slot_provider: Arc<SystemClockSlotProvider>,
    rx: mpsc::Receiver<()>,
) -> Result<()> {
    ReceiverStream::new(rx)
        .for_each_concurrent(MAX_CONCURRENT_SLOTS, |_| async {
            if let Some(slot) = slot_provider.get_last_slot().await {
                log::debug!(
                    "processing slot with number: {}, slot start: {}, processing started at: {}",
                    slot,
                    slot.to_date_time(),
                    Utc::now()
                );
                timeout(
                    tokio::time::Duration::from_secs(RUN_SLOT_LIMIT_SECS),
                    run_single_slot(
                        price_provider.clone(),
                        message_generator.clone(),
                        message_broadcaster.clone(),
                        slot,
                    ),
                )
                .await
                .unwrap_or_else(|_| {
                    log::error!("Hit {}s timeout for slot: {}", RUN_SLOT_LIMIT_SECS, slot);
                    Ok(())
                })
                .unwrap_or_else(|e| {
                    log::error!("Error when running for slot: {} - {:?}", slot, e);
                });
            }
        })
        .await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Assumes that GOFER_CMD env variable is set to the gofer binary
    let mut gofer_cmd = std::env::var("GOFER_CMD")?;
    gofer_cmd.push_str(" prices --norpc ETH/USD");
    log::debug!("Gofer command: {}", gofer_cmd);

    let price_provider = Arc::new(GoferPriceProvider::new(&gofer_cmd));
    log::info!("Initialized price_provider");
    // TODO: Replace with a signature provider that lets the operator use their validator key
    let signature_provider = PrivateKeySignatureProvider::random();
    log::info!("Initialized signature_provider");
    let message_generator = MessageGenerator::new(Box::new(signature_provider));
    log::info!("Initialized message_generator");
    let message_broadcaster = Arc::new(HttpMessageBroadcaster::new()?);
    log::info!("Initialized message_roadcaster");
    // It's hard to have the slot provider be sent across threads, own half of the notification
    // channel, and be able to drop the other half of the channel when the slot provider is
    // dropped. Room for improvement here.
    let (tx, rx) = mpsc::channel(1);
    let slot_provider = SystemClockSlotProvider::new(tx);
    log::info!("Initialized slot_provider");

    run_oracle_node(
        price_provider,
        message_generator,
        message_broadcaster,
        slot_provider,
        rx,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_broadcaster::{json::JsonFileMessageBroadcaster, OracleMessage};
    use signature_provider::SignatureProvider;
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

        let price_provider = Arc::new(GoferPriceProvider::new("cat ./test_data/input.json"));

        let signature_provider = PrivateKeySignatureProvider::random();
        let public_key = signature_provider.get_public_key().unwrap();
        let message_generator = MessageGenerator::new(Box::new(signature_provider));
        let output_files_before = get_output_files();
        let message_broadcaster = Arc::new(
            JsonFileMessageBroadcaster::new(Some("test_data/output".to_string())).unwrap(),
        );

        let (tx, rx) = mpsc::channel(1);
        let slot_provider = SystemClockSlotProvider::new_with_max_count(tx, 1);

        run_oracle_node(
            price_provider,
            message_generator,
            message_broadcaster,
            slot_provider,
            rx,
        )
        .await
        .unwrap();

        let output_files_after = get_output_files();

        let new_output_files = subtract_vecs(&output_files_after, &output_files_before);
        assert_eq!(new_output_files.len(), 1);
        let new_output_file = fs::File::open(new_output_files.get(0).unwrap()).unwrap();
        let oracle_message: OracleMessage = serde_json::from_reader(new_output_file).unwrap();

        assert_eq!(oracle_message.validator_public_key, public_key);
        assert!(oracle_message.interval_inclusion_messages.len() > 100);
    }
}
