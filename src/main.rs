use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::Result;
use futures::stream::Stream;
use ssz::Encode;

mod message_broadcaster;
use message_broadcaster::{log::LogMessageBroadcaster, MessageBroadcaster, PriceMessage};
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, PriceProvider};
mod signature_provider;
use signature_provider::{private_key::PrivateKeySignatureProvider, SignatureProvider};

async fn run_oracle_node(
    price_provider: impl PriceProvider,
    signature_provider: impl SignatureProvider,
    message_broadcaster: impl MessageBroadcaster,
    mut epoch_stream: impl Stream<Item = u64> + std::marker::Unpin,
) -> Result<()> {
    while let Some(epoch_number) = epoch_stream.next().await {
        println!("Reporting price for epoch: {}", epoch_number);
        let price = price_provider.get_price().expect("Error getting price");
        let price_ssz: Vec<u8> = price.as_ssz_bytes();
        let signature = signature_provider.sign(&price_ssz).expect("Error signing");
        let message = PriceMessage { price, signature };
        message_broadcaster.broadcast(message);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let price_provider = GoferPriceProvider::new(None);

    // TODO: Replace with a signature provider that lets the operator use their validator key
    let signature_provider = PrivateKeySignatureProvider::random();

    // TODO: Replace with a broadcaster that reports the results to our server
    let message_broadcaster = LogMessageBroadcaster {};

    // TODO: Replace with stream that returns the current epoch instead of mined blocknumbers
    let provider =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await?;
    // Ensures this program exits after a few blocks blocks for easier testing, in production the stream should run forever
    let num_of_blocks_to_run = 3;
    let block_stream = provider
        .subscribe_blocks()
        .await?
        .take(num_of_blocks_to_run);
    let epoch_stream = block_stream.map(|block| block.number.unwrap().as_u64());

    run_oracle_node(
        price_provider,
        signature_provider,
        message_broadcaster,
        epoch_stream,
    )
    .await?;
    Ok(())
}
