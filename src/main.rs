use eyre::{Result, WrapErr};

mod message_broadcaster;
use message_broadcaster::{http::HttpMessageBroadcaster, MessageBroadcaster};
mod message_generator;
use message_generator::MessageGenerator;
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PriceProvider, PRECISION_FACTOR};
mod signature_provider;
use signature_provider::private_key::PrivateKeySignatureProvider;
mod slot_provider;
use slot_provider::{clock::{GENESIS_SLOT_TIME, SLOT_PERIOD_SECONDS, SystemClockSlotProvider}, Slot, SlotProvider};

async fn run_oracle_node(
    price_provider: impl PriceProvider + std::marker::Send + std::marker::Sync + Clone + 'static,
    message_generator: MessageGenerator,
    message_broadcaster: impl MessageBroadcaster
        + std::marker::Send
        + std::marker::Sync
        + Clone
        + 'static,
    slot_provider: impl SlotProvider,
) -> Result<()> {
    slot_provider
        .run_for_every_slot(
            move |slot: slot_provider::Slot| -> Box<dyn futures::Future<Output = ()> + std::marker::Send + Unpin> {
                let message_broadcaster = message_broadcaster.clone();
                let message_generator = message_generator.clone();
                let price_provider = price_provider.clone();

                Box::new(Box::pin(async move {
                    run_single_slot(
                        price_provider,
                        message_generator,
                        message_broadcaster,
                        slot.clone()
                    ).await.unwrap_or_else(|e| {
                        log::error!("Error when running for slot: {} - {:?}", slot.number, e);
                    });
                }))
            },
        )
        .await
}

async fn run_single_slot(
    price_provider: impl PriceProvider + std::marker::Send + std::marker::Sync + Clone + 'static,
    message_generator: MessageGenerator,
    message_broadcaster: impl MessageBroadcaster
        + std::marker::Send
        + std::marker::Sync
        + Clone
        + 'static,
    slot: Slot,
) -> Result<()> {
    log::info!("Running for slot: {}", slot.number);
    let slot_number = slot.number;
    let slot_start_time = GENESIS_SLOT_TIME + SLOT_PERIOD_SECONDS * slot_number;
    let start_time = chrono::Utc::now().timestamp();
    let lag = start_time - slot_start_time as i64;
    log::info!("Lagging {} seconds behind slot start for slot {}", lag, slot_number);

    let price = price_provider
        .get_price()
        .wrap_err("Failed to get price data")?;
    log::info!(
        "Sucessfully obtained current Eth Price: {:?}",
        price.value as f64 / PRECISION_FACTOR as f64
    );
    let oracle_message = &message_generator
        .generate_oracle_message(price.clone(), slot)
        .wrap_err("Failed to generated signed price message")?;
    log::info!("Sucessfully generated signed price message");
    message_broadcaster
        .broadcast(oracle_message)
        .await
        .wrap_err("Failed to broadcast message")?;
    log::info!("Sucessfully ran for slot: {}", slot_number);

    let end_time = chrono::Utc::now().timestamp();
    let time_elapsed = end_time - start_time;
    log::info!("Time elapsed for slot {} : {} seconds", slot_number, time_elapsed);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut gofer_cmd = std::env::var("GOFER_CMD")?; // Assumes that GOFER_CMD env variable is set
                                                     // to the gofer binary
    gofer_cmd.push_str(" prices --norpc ETH/USD");
    log::debug!("Gofer command: {}", gofer_cmd);

    let price_provider = GoferPriceProvider::new(&gofer_cmd);
    log::info!("Initialized price_provider");
    // TODO: Replace with a signature provider that lets the operator use their validator key
    let signature_provider = PrivateKeySignatureProvider::random();
    log::info!("Initialized signature_provider");
    let message_generator = MessageGenerator::new(Box::new(signature_provider));
    log::info!("Initialized message_generator");
    let http_broadcaster = HttpMessageBroadcaster::new(None)?;
    log::info!("Initialized message_roadcaster");
    // TODO: Replace with a provider that returns every slot number independent of whether it's been mined
    let slot_provider = SystemClockSlotProvider::new();
    log::info!("Initialized slot_provider");

    run_oracle_node(
        price_provider,
        message_generator,
        http_broadcaster,
        slot_provider,
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

        let price_provider = GoferPriceProvider::new("cat ./test_data/input.json");

        let signature_provider = PrivateKeySignatureProvider::random();
        let public_key = signature_provider.get_public_key().unwrap();
        let message_generator = MessageGenerator::new(Box::new(signature_provider));
        let output_files_before = get_output_files();
        let message_broadcaster =
            JsonFileMessageBroadcaster::new(Some("test_data/output".to_string())).unwrap();

        let slot_provider = SystemClockSlotProvider::stop_after_num_slots(1);

        run_oracle_node(
            price_provider,
            message_generator,
            message_broadcaster,
            slot_provider,
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
