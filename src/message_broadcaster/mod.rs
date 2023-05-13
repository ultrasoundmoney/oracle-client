use crate::price_provider::Price;
use bls_signatures::Signature;

pub mod log;

#[derive(Debug)]
pub struct PriceMessage {
    pub price: Price,
    pub signature: Signature,
}

pub trait MessageBroadcaster {
    fn broadcast(&self, msg: PriceMessage);
}
