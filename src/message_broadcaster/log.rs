use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub struct LogMessageBroadcaster {}

impl MessageBroadcaster for LogMessageBroadcaster {
    fn broadcast(&self, msg: OracleMessage) -> Result<()> {
        // log::debug!("Broadcasting message: {:?}", msg);
        Ok(())
    }
}
