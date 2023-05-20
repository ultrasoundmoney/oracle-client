use crate::price_provider::{Price, PriceProvider, PRECISION_FACTOR};
use eyre::Result;
use std::process::Command;

mod types;

pub struct GoferPriceProvider {
    gofer_cmd: String,
}

impl GoferPriceProvider {
    pub fn new(gofer_cmd: &str) -> GoferPriceProvider {
        GoferPriceProvider {
            gofer_cmd: gofer_cmd.to_string(),
        }
    }
}

impl PriceProvider for GoferPriceProvider {
    fn get_price(&self) -> Result<Price> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.gofer_cmd)
            // .arg("ETH/USD")
            .output()?;
        let string_output = String::from_utf8(output.stdout)?;
        let data: types::Root = serde_json::from_str(&string_output)?;
        let value = (data.price * PRECISION_FACTOR as f64) as u64;
        Ok(Price { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    // Basic integration tests mocking out gofer with a static file
    async fn parses_price_correctly() {
        let price_provider = GoferPriceProvider::new("cat ./test_data/input.json");

        let price = price_provider.get_price().unwrap();
        assert_eq!(price.value, 1811093163);
    }
}
