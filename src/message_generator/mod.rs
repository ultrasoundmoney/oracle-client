use eyre::{WrapErr, Result};
use ssz::Encode;

use crate::signature_provider::SignatureProvider;
use crate::slot_provider::Slot;
use crate::message_broadcaster::{OracleMessage, PriceValueMessage, SignedPriceValueMessage, IntervalInclusionMessage, SignedIntervalInclusionMessage};
use crate::price_provider::{Price, PRECISION_FACTOR};

pub const INTERVAL_STEP_DECIMALS: u32 = 2;
pub const INTERVAL_PRECISION_FACTOR: u64 = 10u64.pow(INTERVAL_STEP_DECIMALS);
pub const INTERVAL_SIZE_BASIS_POINTS: u64 = 20; 
pub const ONE_IN_BASIS_POINTS: u64 = 10000;

pub struct MessageGenerator {
    signature_provider: Box<dyn SignatureProvider>,
}

impl MessageGenerator {
    pub fn new(signature_provider: Box<dyn SignatureProvider>) -> MessageGenerator {
        MessageGenerator { signature_provider }
    }

    pub fn generate_oracle_message(&self, price: Price, slot: Slot) -> Result<OracleMessage> {
            let slot_number = slot.number;
            let interval_inclusion_messages = self.generate_signed_interval_inclusion_messages(price.value, slot_number).wrap_err("Failed to generate interval_inclusion_messages")?;
            let value_message = self.generate_signed_price_value_message(price, slot_number).wrap_err("Failed to generate value message")?;
            Ok(OracleMessage {
                value_message,
                interval_inclusion_messages,
            })
    }

    fn get_upper_bound(&self, price_value: u64) -> u64 {
        self.convert_precision(price_value * (ONE_IN_BASIS_POINTS + INTERVAL_SIZE_BASIS_POINTS)  / ONE_IN_BASIS_POINTS)
    }

    fn get_lower_bound(&self, price_value: u64) -> u64 {
        self.convert_precision(price_value * (ONE_IN_BASIS_POINTS - INTERVAL_SIZE_BASIS_POINTS) / ONE_IN_BASIS_POINTS)
    }

    fn convert_precision(&self, price_value: u64) -> u64 {
        price_value * INTERVAL_PRECISION_FACTOR / PRECISION_FACTOR
    }


    fn generate_signed_interval_inclusion_messages(&self, price_value: u64, slot_number: u64) -> Result<Vec<SignedIntervalInclusionMessage>> {
        let upper_bound = self.get_upper_bound(price_value);
        let lower_bound = self.get_lower_bound(price_value);
        log::debug!("Generating interval messages from {} to {}", lower_bound, upper_bound);

        let interval_values = lower_bound..upper_bound;
        log::debug!("Generating messages for {} number of interval_values", interval_values.size_hint().0);
        let interval_inclusion_messages = interval_values
            .map(|value| IntervalInclusionMessage {
                value,
                interval_size: INTERVAL_PRECISION_FACTOR,
                slot_number,
            })
            .collect::<Vec<IntervalInclusionMessage>>();

        log::debug!("Signing {} number fo interval_inclusion_messages", interval_inclusion_messages.len());

        interval_inclusion_messages
            .into_iter()
            .map(|interval_inclusion_message| {
                let interval_inclusion_message_ssz = interval_inclusion_message.as_ssz_bytes();
                let interval_inclusion_message_signature = self.signature_provider.sign(&interval_inclusion_message_ssz).wrap_err("Failed to sign serialized interval inclusion message")?;
                Ok(SignedIntervalInclusionMessage {
                    message: interval_inclusion_message,
                    signature: interval_inclusion_message_signature,
                })
            })
            .collect()
    }

    fn generate_signed_price_value_message(&self, price: Price, slot_number: u64) -> Result<SignedPriceValueMessage> {
            let price_value_message = PriceValueMessage{
                price: price.clone(),
                slot_number,
            };
            let price_value_ssz = price_value_message.as_ssz_bytes();
            let price_value_signature = self.signature_provider.sign(&price_value_ssz).wrap_err("Failed to sign serialized price value message")?;
            Ok(SignedPriceValueMessage {
                message: price_value_message,
                signature: price_value_signature,
            })
    }
}
