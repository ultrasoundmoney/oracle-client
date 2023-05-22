use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub const SERVER_URL: &str = "http://localhost:3000";

pub struct HttpMessageBroadcaster {
    server_url: String,
}

impl HttpMessageBroadcaster {
    pub fn new(server_url: Option<String>) -> Result<HttpMessageBroadcaster> {
        // Create directory if it doesn't exist yet
        let server_url = match server_url {
            Some(path) => path,
            None => String::from(SERVER_URL),
        };
        Ok(HttpMessageBroadcaster { server_url })
    }
}

impl MessageBroadcaster for HttpMessageBroadcaster {

    #[tokio::main]
    async fn broadcast(&self, msg: OracleMessage) -> Result<()> {
        let client = reqwest::Client::new();
        log::debug!("Sending message to server at: {:}", self.server_url);
        let response = client
            .post(&self.server_url)
            .json(&msg)
            .send()
            .await
            .map_err(|e| eyre::eyre!("Error sending message: {}", e))?;
        log::debug!("Response: {:?}", response);
        Ok(())
    }
}
