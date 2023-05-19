use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub const DIRECTORY_PATH: &str = "test_messages";

pub struct JsonFileMessageBroadcaster {
    directory_path: String,
}

impl JsonFileMessageBroadcaster {
    pub fn new() -> Result<JsonFileMessageBroadcaster> {
        // Create directory if it doesn't exist yet
        std::fs::create_dir_all(DIRECTORY_PATH)?;
        Ok(JsonFileMessageBroadcaster {
            directory_path: String::from(DIRECTORY_PATH),
        })
    }
}

impl MessageBroadcaster for JsonFileMessageBroadcaster {
    fn broadcast(&self, msg: OracleMessage) -> Result<()> {
        let file_name = format!(
            "{}/{}.json",
            self.directory_path, msg.value_message.message.slot_number
        );
        log::debug!("Writing message to file: {}", file_name);
        let file = std::fs::File::create(file_name)?;
        serde_json::to_writer_pretty(file, &msg)?;
        Ok(())
    }
}
