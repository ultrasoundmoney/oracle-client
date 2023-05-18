use crate::price_provider::{Price, PriceProvider, PRECISION_FACTOR};
use eyre::Result;
use std::process::Command;

mod types;

pub struct GoferPriceProvider {
    gofer_cmd: String,
}

impl GoferPriceProvider {
    pub fn new(gofer_cmd: Option<&str>) -> Result<GoferPriceProvider> {
        match gofer_cmd {
            Some(cmd) => Ok(GoferPriceProvider {
                gofer_cmd: cmd.to_string(),
            }),
            None => Ok(GoferPriceProvider {
                gofer_cmd: std::env::var("GOFER_CMD")?
            }),
        }
    }
}

impl PriceProvider for GoferPriceProvider {
    fn get_price(&self) -> Result<Price> {
        let output = Command::new(&self.gofer_cmd)
            .arg("prices")
            .arg("--norpc")
            .arg("ETH/USD")
            .output()?;
        let string_output = String::from_utf8(output.stdout)?;
        let data: types::Root = serde_json::from_str(&string_output)?;
        let value = (data.price * PRECISION_FACTOR as f64) as u64;
        Ok(Price { value })
    }
}
