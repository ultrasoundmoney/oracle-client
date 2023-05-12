use ssz_derive::{Decode, Encode};

pub mod gofer;

// SSZ serialization of float is non-trivial so we need to convert to u64 for now
// TODO: See if there is a way to ssz encode a float
pub const PRECISION_DECIMALS: u32 = 6;
pub const PRECISION_FACTOR: u64 = 10u64.pow(PRECISION_DECIMALS);

#[derive(Debug, Encode, Decode)]
pub struct Price {
    pub value: u64, // TODO: Check if we need to add further info here such as timestamp
}

pub trait PriceProvider {
    fn get_price(&self) -> Option<Price>;
}
