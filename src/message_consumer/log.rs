use crate::message_consumer::{MessageConsumer, PriceMessage};

pub struct LogMessageConsumer {
}

impl MessageConsumer for LogMessageConsumer {
    fn consume_message(&self, msg: PriceMessage) {
        println!("Consuming message: {:?}", msg);
    }
}

