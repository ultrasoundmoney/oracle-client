use ssz::{Decode, Encode};

mod message_consumer;
use message_consumer::{log::LogMessageConsumer, MessageConsumer, PriceMessage};
mod price_provider;
use price_provider::{Price, PriceProvider, gofer::GoferPriceProvider};
mod signature_provider;
use signature_provider::{SignatureProvider, private_key::PrivateKeySignatureProvider};



fn main() {
    let gofer = GoferPriceProvider::new(None);
    let price = gofer.get_price().expect("Error getting price");
    let price_ssz: Vec<u8> = price.as_ssz_bytes();
    

    let signature_provider = PrivateKeySignatureProvider::random();
    let signature = signature_provider.sign(&price_ssz).expect("Error signing");
    

    let message_consumer = LogMessageConsumer {};
    let message = PriceMessage {
        price,
        signature,
    };
    message_consumer.consume_message(message);

}
