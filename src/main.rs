use eyre::{Result, WrapErr};

mod message_broadcaster;
use message_broadcaster::{json::JsonFileMessageBroadcaster, MessageBroadcaster};
mod message_generator;
use message_generator::MessageGenerator;
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PriceProvider, PRECISION_FACTOR};
mod signature_provider;
use signature_provider::private_key::PrivateKeySignatureProvider;
mod slot_provider;
use slot_provider::{mined_blocks::MinedBlocksSlotProvider, SlotProvider};

async fn run_oracle_node(
    price_provider: impl PriceProvider + 'static,
    message_generator: MessageGenerator,
    message_broadcaster: impl MessageBroadcaster + 'static,
    slot_provider: impl SlotProvider,
) -> Result<()> {
    slot_provider
        .run_for_every_slot(move |slot| -> Result<()> {
            let price = price_provider
                .get_price()
                .wrap_err("Failed to get price data")?;
            log::info!(
                "Sucessfully obtained current Eth Price: {:?}",
                price.value as f64 / PRECISION_FACTOR as f64
            );
            let oracle_message = message_generator
                .generate_oracle_message(price, slot)
                .wrap_err("Failed to generated signed price message")?;
            log::info!("Sucessfully generated signed price message");
            message_broadcaster
                .broadcast(oracle_message)
                .wrap_err("Failed to broadcast message")?;
            Ok(())
        })
        .await
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
    // TODO: Replace with a broadcaster that reports the results to our server
    let message_broadcaster = JsonFileMessageBroadcaster::new(None)?;
    log::info!("Initialized message_broadcaster");
    // TODO: Replace with a provider that returns every slot number independent of whether it's been mined
    let slot_provider = MinedBlocksSlotProvider::new(None).await?;
    log::info!("Initialized slot_provider");

    run_oracle_node(
        price_provider,
        message_generator,
        message_broadcaster,
        slot_provider,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_broadcaster::OracleMessage;
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
        let price_provider = GoferPriceProvider::new("cat /Users/christian/PersonalProjects/ultrasoundmoney/oracle-client/test_data/input.json");

        let signature_provider = PrivateKeySignatureProvider::random();
        let public_key = signature_provider.get_public_key().unwrap();
        let message_generator = MessageGenerator::new(Box::new(signature_provider));
        let output_files_before = get_output_files();
        let message_broadcaster =
            JsonFileMessageBroadcaster::new(Some("test_data/output".to_string())).unwrap();

        let slot_provider = MinedBlocksSlotProvider::new(Some(1)).await.unwrap();

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
