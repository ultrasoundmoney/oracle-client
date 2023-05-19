use eyre::{WrapErr, Result};
use ssz::Encode;

use crate::signature_provider::SignatureProvider;
use crate::message_broadcaster::PriceMessage;
use crate::price_provider::Price;

pub struct MessageGenerator {
    signature_provider: Box<dyn SignatureProvider>,
}

impl MessageGenerator {
    pub fn new(signature_provider: Box<dyn SignatureProvider>) -> MessageGenerator {
        MessageGenerator { signature_provider }
    }

    pub fn generate_signed_price_message(&self, price: Price) -> Result<PriceMessage> {
            let price_ssz: Vec<u8> = price.as_ssz_bytes();
            let signature = self.signature_provider
                .sign(&price_ssz)
                .wrap_err("Failed to sign serialized price data")?;
            Ok(PriceMessage {
                price,
                signature,
            })
    }
}
