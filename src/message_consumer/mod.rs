use crate::price_provider::Price;
use bls_signatures::Signature;

pub mod log;

#[derive(Debug)]
pub struct PriceMessage {
    pub price: Price,
    pub signature: Signature,
}

pub trait MessageConsumer {
    fn consume_message(&self, msg: PriceMessage);
}
