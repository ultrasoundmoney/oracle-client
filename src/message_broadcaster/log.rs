use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub struct LogMessageBroadcaster {}

impl MessageBroadcaster for LogMessageBroadcaster {
    fn broadcast(&self, msg: OracleMessage) -> Box<dyn futures::Future<Output = Result<()>> + Unpin + '_> {
        log::debug!("Broadcasting message: {:?}", msg);
        Box::new(futures::future::ready(Ok(())))
    }
}

impl Clone for LogMessageBroadcaster {
    fn clone(&self) -> Self {
        LogMessageBroadcaster {}
    }
}
