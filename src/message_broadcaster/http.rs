use async_trait::async_trait;
use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};


pub struct HttpMessageBroadcaster {
    server_url: String,
}

impl HttpMessageBroadcaster {
    pub fn new(server_url: Option<String>) -> Result<HttpMessageBroadcaster> {
        // Create directory if it doesn't exist yet

        let server_url = match server_url {
            Some(path) => path,
            None =>  {  std::env::var("SERVER_URL")? },
        };
        Ok(HttpMessageBroadcaster { server_url })
    }

    async fn send_request(&self, msg: OracleMessage) -> Result<()> {
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
    async fn broadcast(&self, msg: OracleMessage) -> Result<()> {
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
