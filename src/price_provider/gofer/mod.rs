use std::process::Command;
use crate::price_provider::{Price, PriceProvider};

mod types;

pub struct GoferPriceProvider {
    gofer_cmd: String
}

impl GoferPriceProvider {
    pub fn new(gofer_cmd: Option<&str>) -> GoferPriceProvider {
        match gofer_cmd {
            Some(cmd) => GoferPriceProvider {
                gofer_cmd: cmd.to_string()
            },
            None => GoferPriceProvider {
                gofer_cmd: std::env::var("GOFER_CMD").expect("Neither GOFER_CMD env variable nor gofer_cmd argument was provided")
            }
        }
    }
}

impl PriceProvider for GoferPriceProvider {
    fn get_price(&self) -> Option<Price> {
        let output = Command::new(&self.gofer_cmd)
            .arg("prices")
            .arg("--norpc")
            .arg("ETH/USD")
            .output()
            .expect("failed to execute process");
        let string_output = String::from_utf8(output.stdout).expect("Error decoding");
        println!("string_output: {}", string_output);
        let data: types::Root = serde_json::from_str(&string_output).expect("Error parsing");
        let value = data.price;
        Some(Price {
            value
        })
    }
}
