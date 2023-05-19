use eyre::{WrapErr, Result};

mod message_broadcaster;
use message_broadcaster::{log::LogMessageBroadcaster, MessageBroadcaster};
mod message_generator;
use message_generator::MessageGenerator;
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PRECISION_FACTOR, PriceProvider};
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
            let price = price_provider.get_price().wrap_err("Failed to get price data")?;
            log::info!("Sucessfully obtained current Eth Price: {:?}", price.value as f64 / PRECISION_FACTOR as f64);
            let oracle_message = message_generator.generate_oracle_message(price, slot).wrap_err("Failed to generated signed price message")?;
            log::info!("Sucessfully generated signed price message");
            log::debug!("signed_price_message: {:?}", oracle_message);
            message_broadcaster.broadcast(oracle_message).wrap_err("Failed to broadcast message")?;
            Ok(())
        })
        .await
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let price_provider = GoferPriceProvider::new(None)?;
    log::info!("Initialized price_provider");
    // TODO: Replace with a signature provider that lets the operator use their validator key
    let signature_provider = PrivateKeySignatureProvider::random();
    log::info!("Initialized signature_provider");
    let message_generator = MessageGenerator::new(Box::new(signature_provider));
    log::info!("Initialized message_generator");
    // TODO: Replace with a broadcaster that reports the results to our server
    let message_broadcaster = LogMessageBroadcaster {};
    log::info!("Initialized message_broadcaster");
    // TODO: Replace with a provider that returns every slot number independent of whether it's been mined
    let slot_provider = MinedBlocksSlotProvider::new().await?;
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
