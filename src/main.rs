use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::Result;
use ssz::{Decode, Encode};

mod message_consumer;
use message_consumer::{log::LogMessageConsumer, MessageConsumer, PriceMessage};
mod price_provider;
use price_provider::{gofer::GoferPriceProvider, Price, PriceProvider};
mod signature_provider;
use signature_provider::{private_key::PrivateKeySignatureProvider, SignatureProvider};

#[tokio::main]
async fn main() -> Result<()> {
    let provider =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await?;
    let mut stream = provider.subscribe_blocks().await?;

    let gofer = GoferPriceProvider::new(None);
    let signature_provider = PrivateKeySignatureProvider::random();

    let message_consumer = LogMessageConsumer {};
    while let Some(block) = stream.next().await {
        println!(
            "Ts: {:?}, block number: {} -> {:?}",
            block.timestamp,
            block.number.unwrap(),
            block.hash.unwrap()
        );
        let price = gofer.get_price().expect("Error getting price");
        let price_ssz: Vec<u8> = price.as_ssz_bytes();
        let signature = signature_provider.sign(&price_ssz).expect("Error signing");
        let message = PriceMessage { price, signature };
        message_consumer.consume_message(message);
    }

    Ok(())
}
