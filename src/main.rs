mod attestation_scheduler;
mod message_broadcaster;
mod slot;

use eyre::Result;
use message_broadcaster::http::HttpMessageBroadcaster;
mod message_generator;
use message_generator::MessageGenerator;
mod price_provider;
use price_provider::gofer::GoferPriceProvider;
mod signature_provider;
use signature_provider::private_key::PrivateKeySignatureProvider;

use crate::attestation_scheduler::SystemClockAttestationScheduler;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let gofer_url = std::env::var("GOFER_URL").unwrap_or("http://localhost:9200/price".to_string());
    log::debug!("Gofer URL: {}", gofer_url);
    let price_provider = GoferPriceProvider::new(gofer_url.as_str());
    log::info!("Initialized price_provider");
    // TODO: Replace with a signature provider that lets the operator use their validator key
    let signature_provider = PrivateKeySignatureProvider::random();
    log::info!("Initialized signature_provider");
    let message_generator = MessageGenerator::new(Box::new(signature_provider));
    log::info!("Initialized message_generator");
    let message_broadcaster = HttpMessageBroadcaster::new()?;
    log::info!("Initialized message_roadcaster");

    let attestation_scheduler = SystemClockAttestationScheduler::new(
        message_broadcaster,
        message_generator,
        price_provider,
        None,
    );

    attestation_scheduler.run().await;

    Ok(())
}
