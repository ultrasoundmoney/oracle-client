use crate::price_provider::gofer::types::GoferPriceRequest;
use crate::price_provider::{Price, PriceProvider, PRECISION_FACTOR};
use async_trait::async_trait;
use eyre::Result;

mod types;

pub struct GoferPriceProvider {
    gofer_url: String,
    pair: String,
}

impl GoferPriceProvider {
    pub fn new(gofer_url: &str) -> GoferPriceProvider {
        GoferPriceProvider {
            gofer_url: gofer_url.to_string(),
            pair: "ETH/USD".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_pair(gofer_url: &str, pair: String) -> GoferPriceProvider {
        GoferPriceProvider {
            gofer_url: gofer_url.to_string(),
            pair,
        }
    }

    async fn request_prices(&self) -> Result<String> {
        let msg = GoferPriceRequest {
            pair: self.pair.clone(),
        };
        let client = reqwest::Client::new();
        log::debug!("Getting message from gofer at: {:}", self.gofer_url);
        let response = client
            .post(&self.gofer_url)
            .json(&msg)
            .send()
            .await
            .map_err(|e| eyre::eyre!("Error sending message: {}", e))?;
        log::debug!("Response: {:?}", response);
        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(eyre::eyre!(
                "Non-Success response when submitting oracle message: {:?}",
                response
            ))
        }
    }
}

#[async_trait]
impl PriceProvider for GoferPriceProvider {
    async fn get_price(&self) -> Result<Price> {
        let output = self.request_prices().await?;
        let data: types::Root = serde_json::from_str(&output)?;
        let value = (data.price * PRECISION_FACTOR as f64) as u64;
        Ok(Price { value })
    }
}

impl Clone for GoferPriceProvider {
    fn clone(&self) -> Self {
        GoferPriceProvider {
            gofer_url: self.gofer_url.clone(),
            pair: self.pair.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    // Basic integration tests mocking out gofer with a static file
    async fn parses_price_correctly() {
        let mut server = mockito::Server::new();

        let response_json = r#"{
                    "type":"aggregator",
                    "base":"ETH",
                    "quote":"USD",
                    "price":1953,
                    "bid":1953,
                    "ask":1952,
                    "vol24h":0,
                    "ts":"2023-07-04T15:55:48Z",
                    "prices":[]
                }"#;

        // Create a mock
        let mock = server
            .mock("POST", "/price")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_json)
            .create();

        let url = format!("{}{}", server.url().as_str(), "/price");
        let price_provider = GoferPriceProvider::new(url.as_str());

        let price = price_provider.get_price().await.unwrap();
        assert_eq!(price.value, 1953000000);

        mock.assert();
    }
}
