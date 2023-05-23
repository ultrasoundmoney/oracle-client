use eyre::Result;

use crate::message_broadcaster::{MessageBroadcaster, OracleMessage};

pub const DIRECTORY_PATH: &str = "test_messages";

pub struct JsonFileMessageBroadcaster {
    directory_path: String,
}

impl JsonFileMessageBroadcaster {
    pub fn new(directory_path: Option<String>) -> Result<JsonFileMessageBroadcaster> {
        // Create directory if it doesn't exist yet
        let directory_path = match directory_path {
            Some(path) => path,
            None => String::from(DIRECTORY_PATH),
        };
        std::fs::create_dir_all(&directory_path)?;
        Ok(JsonFileMessageBroadcaster { directory_path })
    }

    fn write_file(&self, msg: OracleMessage) -> Result<()> {
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

impl MessageBroadcaster for JsonFileMessageBroadcaster {
    fn broadcast(&self, msg: OracleMessage) -> Box<dyn futures::Future<Output = Result<()>> + Unpin> {
        Box::new(futures::future::ready(self.write_file(msg)))
    }
}

impl Clone for JsonFileMessageBroadcaster {
    fn clone(&self) -> Self {
        JsonFileMessageBroadcaster {
            directory_path: self.directory_path.clone(),
        }
    }
}
