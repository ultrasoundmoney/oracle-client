use eyre::Result;
use ssz_derive::{Decode, Encode};

use crate::price_provider::Price;
use bls::Signature;

pub mod log;

#[derive(Debug)]
pub struct OracleMessage {
    pub value_message: SignedPriceValueMessage,
    pub interval_inclusion_messages: Vec<SignedIntervalInclusionMessage>,
}

#[derive(Debug, Decode, Encode)]
pub struct PriceValueMessage {
    pub price: Price,
    pub slot_number: u64,
}

#[derive(Debug)]
pub struct SignedPriceValueMessage {
    pub message: PriceValueMessage,
    pub signature: Signature,
}

#[derive(Debug, Decode, Encode)]
pub struct IntervalInclusionMessage {
    pub value: u64,
    pub interval_size: u64,
    pub slot_number: u64,
}

#[derive(Debug)]
pub struct SignedIntervalInclusionMessage {
    pub message: IntervalInclusionMessage,
    pub signature: Signature,
}

pub trait MessageBroadcaster {
    fn broadcast(&self, msg: OracleMessage) -> Result<()>;
}
