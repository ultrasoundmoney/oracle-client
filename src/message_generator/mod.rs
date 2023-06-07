use eyre::{Result, WrapErr};
use ssz::Encode;

use crate::message_broadcaster::{
    IntervalInclusionMessage, OracleMessage, PriceValueMessage, SignedIntervalInclusionMessage,
    SignedPriceValueMessage,
};
use crate::price_provider::{Price, PRECISION_FACTOR};
use crate::signature_provider::SignatureProvider;
use crate::slot_provider::Slot;

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
        let interval_inclusion_messages = self
            .generate_signed_interval_inclusion_messages(price.value, slot_number)
            .wrap_err("Failed to generate interval_inclusion_messages")?;
        let value_message = self
            .generate_signed_price_value_message(price, slot_number)
            .wrap_err("Failed to generate value message")?;
        let validator_public_key = self
            .signature_provider
            .get_public_key()
            .wrap_err("Failed to get public key")?;
        Ok(OracleMessage {
            value_message,
            interval_inclusion_messages,
            validator_public_key,
        })
    }

    fn get_upper_bound(&self, price_value: u64) -> u64 {
        self.convert_precision(
            price_value * (ONE_IN_BASIS_POINTS + INTERVAL_SIZE_BASIS_POINTS) / ONE_IN_BASIS_POINTS,
        )
    }

    fn get_lower_bound(&self, price_value: u64) -> u64 {
        self.convert_precision(
            price_value * (ONE_IN_BASIS_POINTS - INTERVAL_SIZE_BASIS_POINTS) / ONE_IN_BASIS_POINTS,
        )
    }

    fn convert_precision(&self, price_value: u64) -> u64 {
        price_value * INTERVAL_PRECISION_FACTOR / PRECISION_FACTOR
    }

    fn generate_signed_interval_inclusion_messages(
        &self,
        price_value: u64,
        slot_number: u64,
    ) -> Result<Vec<SignedIntervalInclusionMessage>> {
        let upper_bound = self.get_upper_bound(price_value);
        let lower_bound = self.get_lower_bound(price_value);
        log::debug!(
            "Generating interval messages from {} to {}",
            lower_bound,
            upper_bound
        );

        let interval_values = lower_bound..upper_bound;
        log::debug!(
            "Generating messages for {} number of interval_values",
            interval_values.size_hint().0
        );
        let interval_inclusion_messages = interval_values
            .map(|value| IntervalInclusionMessage {
                value,
                interval_size: INTERVAL_SIZE_BASIS_POINTS,
                slot_number,
            })
            .collect::<Vec<IntervalInclusionMessage>>();

        log::debug!(
            "Signing {} number fo interval_inclusion_messages",
            interval_inclusion_messages.len()
        );

        interval_inclusion_messages
            .into_iter()
            .map(|interval_inclusion_message| {
                let interval_inclusion_message_ssz = interval_inclusion_message.as_ssz_bytes();
                let interval_inclusion_message_signature = self
                    .signature_provider
                    .sign(&interval_inclusion_message_ssz)
                    .wrap_err("Failed to sign serialized interval inclusion message")?;
                Ok(SignedIntervalInclusionMessage {
                    message: interval_inclusion_message,
                    signature: interval_inclusion_message_signature,
                })
            })
            .collect()
    }

    fn generate_signed_price_value_message(
        &self,
        price: Price,
        slot_number: u64,
    ) -> Result<SignedPriceValueMessage> {
        let price_value_message = PriceValueMessage { price, slot_number };
        let price_value_ssz = price_value_message.as_ssz_bytes();
        let price_value_signature = self
            .signature_provider
            .sign(&price_value_ssz)
            .wrap_err("Failed to sign serialized price value message")?;
        Ok(SignedPriceValueMessage {
            message: price_value_message,
            signature: price_value_signature,
        })
    }
}

impl Clone for MessageGenerator {
    fn clone(&self) -> Self {
        MessageGenerator {
            signature_provider: self.signature_provider.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrivateKeySignatureProvider;

    #[tokio::test]
    async fn generates_correct_price_value_messsage() {
        let signature_provider = PrivateKeySignatureProvider::random();
        let message_generator = MessageGenerator::new(signature_provider.clone());
        let price = Price {
            value: 1000 * PRECISION_FACTOR,
        };
        let slot = Slot { number: 1 };

        let oracle_message = message_generator
            .generate_oracle_message(price.clone(), slot.clone())
            .unwrap();

        assert!(oracle_message
            .value_message
            .message
            .price
            .value
            .eq(&price.value));

        assert!(oracle_message
            .value_message
            .message
            .slot_number
            .eq(&slot.number));
        assert!(oracle_message.value_message.signature.verify(
            &oracle_message.validator_public_key,
            signature_provider
                .get_message_digest(&oracle_message.value_message.message.as_ssz_bytes())
        ));
        assert!(oracle_message
            .validator_public_key
            .to_string()
            .eq(&signature_provider.get_public_key().unwrap().to_string()));
    }

    #[tokio::test]
    async fn generates_correct_inclusion_messages() {
        let signature_provider = PrivateKeySignatureProvider::random();
        let message_generator = MessageGenerator::new(signature_provider.clone());
        let price = Price {
            value: 1000 * PRECISION_FACTOR,
        };
        let slot = Slot { number: 1 };

        let oracle_message = message_generator
            .generate_oracle_message(price.clone(), slot.clone())
            .unwrap();

        assert_eq!(oracle_message.interval_inclusion_messages.len(), 400);

        assert_eq!(
            oracle_message.interval_inclusion_messages[0].message.value,
            998 * INTERVAL_PRECISION_FACTOR
        );
        assert_eq!(
            oracle_message
                .interval_inclusion_messages
                .last()
                .unwrap()
                .message
                .value,
            1002 * INTERVAL_PRECISION_FACTOR - 1
        );

        for (i, interval_inclusion_message) in oracle_message
            .interval_inclusion_messages
            .iter()
            .enumerate()
        {
            assert!(interval_inclusion_message.signature.verify(
                &oracle_message.validator_public_key,
                signature_provider
                    .get_message_digest(&interval_inclusion_message.message.as_ssz_bytes())
            ));
            assert_eq!(interval_inclusion_message.message.slot_number, slot.number);
            assert_eq!(
                interval_inclusion_message.message.interval_size,
                INTERVAL_SIZE_BASIS_POINTS
            );
            if i > 0 {
                assert_eq!(
                    interval_inclusion_message.message.value,
                    oracle_message.interval_inclusion_messages[i - 1]
                        .message
                        .value
                        + 1
                );
            }
        }
    }
}
