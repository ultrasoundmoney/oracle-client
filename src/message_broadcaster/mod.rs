use eyre::Result;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

use crate::price_provider::Price;
use bls::{PublicKey, Signature};

pub mod json;
pub mod log;

#[derive(Debug, Serialize, Deserialize)]
pub struct OracleMessage {
    pub value_message: SignedPriceValueMessage,
    pub interval_inclusion_messages: Vec<SignedIntervalInclusionMessage>,
    pub validator_public_key: PublicKey,
}

#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct PriceValueMessage {
    pub price: Price,
    pub slot_number: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedPriceValueMessage {
    pub message: PriceValueMessage,
    pub signature: Signature,
}

#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct IntervalInclusionMessage {
    pub value: u64,
    pub interval_size: u64,
    pub slot_number: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedIntervalInclusionMessage {
    pub message: IntervalInclusionMessage,
    pub signature: Signature,
}

pub trait MessageBroadcaster {
    fn broadcast(&self, msg: OracleMessage) -> Result<()>;
}
