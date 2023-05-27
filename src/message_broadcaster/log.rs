use async_trait::async_trait;
use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub struct LogMessageBroadcaster {}

#[async_trait]
impl MessageBroadcaster for LogMessageBroadcaster {
    async fn broadcast(&self, msg: &OracleMessage) -> Result<()> {
        log::debug!("Broadcasting message: {:?}", msg);
        Ok(())
    }
}

impl Clone for LogMessageBroadcaster {
    fn clone(&self) -> Self {
        LogMessageBroadcaster {}
    }
}
