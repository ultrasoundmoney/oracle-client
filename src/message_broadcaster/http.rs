use async_trait::async_trait;
use eyre::{Context, Result};

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub struct HttpMessageBroadcaster {
    server_url: String,
}

impl HttpMessageBroadcaster {
    pub fn new() -> Result<HttpMessageBroadcaster> {
        let server_url = std::env::var("SERVER_URL").context(
            "expect SERVER_URL in env when no server_url is given to HttpMessageBroadcaster",
        )?;
        Ok(HttpMessageBroadcaster { server_url })
    }

    #[cfg(test)]
    pub fn new_with_url(server_url: &str) -> HttpMessageBroadcaster {
        HttpMessageBroadcaster {
            server_url: server_url.to_string(),
        }
    }

    async fn send_request(&self, msg: &OracleMessage) -> Result<()> {
        let client = reqwest::Client::new();
        log::debug!("Sending message to server at: {:}", self.server_url);
        let response = client
            .post(&self.server_url)
            .json(&msg)
            .send()
            .await
            .map_err(|e| eyre::eyre!("Error sending message: {}", e))?;
        log::debug!("Response: {:?}", response);
        if response.status().is_success() {
            Ok(())
        } else {
            Err(eyre::eyre!(
                "Non-Success response when submitting oracle message: {:?}",
                response
            ))
        }
    }
}

#[async_trait]
impl MessageBroadcaster for HttpMessageBroadcaster {
    async fn broadcast(&self, msg: &OracleMessage) -> Result<()> {
        self.send_request(msg).await?;
        Ok(())
    }
}

impl Clone for HttpMessageBroadcaster {
    fn clone(&self) -> Self {
        HttpMessageBroadcaster {
            server_url: self.server_url.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use mockito::Matcher;

    use crate::{
        message_generator::MessageGenerator, price_provider::Price,
        signature_provider::private_key::PrivateKeySignatureProvider, slot::Slot,
    };

    use super::*;

    #[tokio::test]
    async fn test_http_message_broadcaster() -> Result<()> {
        let mut server = mockito::Server::new();

        let broadcaster = HttpMessageBroadcaster::new_with_url(&server.url());

        let signature_provider = PrivateKeySignatureProvider::random();
        let message = MessageGenerator::new(Box::new(signature_provider))
            .generate_oracle_message(Price { value: 10 }, Slot(1))?;

        let mock = server
            .mock("POST", "/")
            .match_body(Matcher::Json(serde_json::to_value(&message)?))
            .with_status(200)
            .create();

        broadcaster.broadcast(&message).await?;

        mock.assert();

        Ok(())
    }
}
