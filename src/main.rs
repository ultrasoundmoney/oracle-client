use eyre::Result;
use ssz::Encode;

mod message_broadcaster;
use message_broadcaster::{log::LogMessageBroadcaster, MessageBroadcaster, PriceMessage};
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PriceProvider};
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
        .run_for_every_slot(move |slot_number| {
            println!("Reporting price for slot: {:?}", slot_number);
            let price = price_provider.get_price().expect("Error getting price");
            let price_ssz: Vec<u8> = price.as_ssz_bytes();
            let signature = signature_provider.sign(&price_ssz).expect("Error signing");
            let message = PriceMessage { price, signature };
            message_broadcaster.broadcast(message);
        })
        .await
}

#[tokio::main]
async fn main() -> Result<()> {
    let price_provider = GoferPriceProvider::new(None);
    // TODO: Replace with a signature provider that lets the operator use their validator key
    let signature_provider = PrivateKeySignatureProvider::random();
    // TODO: Replace with a broadcaster that reports the results to our server
    let message_broadcaster = LogMessageBroadcaster {};
    // TODO: Replace with a provider that returns every slot number independent of whether it's been mined
    let slot_provider = MinedBlocksSlotProvider::new().await;

    run_oracle_node(
        price_provider,
        signature_provider,
        message_broadcaster,
        slot_provider,
    )
    .await?;

    Ok(())
}
