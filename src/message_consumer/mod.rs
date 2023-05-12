use bls_signatures::Signature;
use crate::price_provider::Price;

pub mod log;

#[derive(Debug)]
pub struct PriceMessage {
    pub price: Price,
    pub signature: Signature,
}

pub trait MessageConsumer {
    fn consume_message(&self, msg: PriceMessage);
}
