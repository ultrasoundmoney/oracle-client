use crate::message_broadcaster::{MessageBroadcaster, PriceMessage};

pub struct LogMessageBroadcaster {}

impl MessageBroadcaster for LogMessageBroadcaster {
    fn broadcast(&self, msg: PriceMessage) {
        println!("Broadcasting message: {:?}", msg);
    }
}
