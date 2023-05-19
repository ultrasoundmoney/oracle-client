use eyre::{Result, WrapErr};
use ssz::Encode;

mod message_broadcaster;
use message_broadcaster::{log::LogMessageBroadcaster, MessageBroadcaster, PriceMessage};
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PriceProvider, PRECISION_FACTOR};
mod signature_provider;
use signature_provider::{private_key::PrivateKeySignatureProvider, SignatureProvider};
mod slot_provider;
use slot_provider::{mined_blocks::MinedBlocksSlotProvider, SlotProvider};

async fn run_oracle_node(
    price_provider: impl PriceProvider + 'static,
    signature_provider: impl SignatureProvider + 'static,
    message_broadcaster: impl MessageBroadcaster + 'static,
    slot_provider: impl SlotProvider,
) -> Result<()> {
    slot_provider
        .run_for_every_slot(move |_slot| -> Result<()> {
            let price = price_provider
                .get_price()
                .wrap_err("Failed to get price data")?;
            log::info!(
                "Sucessfully obtained current Eth Price: {:?}",
                price.value as f64 / PRECISION_FACTOR as f64
            );
            let price_ssz: Vec<u8> = price.as_ssz_bytes();
            log::debug!("Succesfully serialized price data: {:?}", price_ssz);
            let signature = signature_provider
                .sign(&price_ssz)
                .wrap_err("Failed to sign serialized price data")?;
            log::debug!("Succesfully signed serialized prize data: {:?}", signature);
            let message = PriceMessage { price, signature };
            message_broadcaster
                .broadcast(message)
                .wrap_err("Failed to broadcast message")?;
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
    // TODO: Replace with a broadcaster that reports the results to our server
    let message_broadcaster = LogMessageBroadcaster {};
    log::info!("Initialized message_broadcaster");
    // TODO: Replace with a provider that returns every slot number independent of whether it's been mined
    let slot_provider = MinedBlocksSlotProvider::new().await?;
    log::info!("Initialized slot_provider");

    run_oracle_node(
        price_provider,
        signature_provider,
        message_broadcaster,
        slot_provider,
    )
    .await?;

    Ok(())
}
