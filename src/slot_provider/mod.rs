use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use eyre::Result;
use futures::Future;
use tokio::time::{Duration};

pub mod clock;
pub mod mined_blocks;

// Dec-01-2020 12:00:23 UTC
pub const GENESIS_SLOT_TIME: u64 = 1606824023;
pub const SLOT_PERIOD_SECONDS: u64 = 12;

#[derive(Clone, Debug)]
pub struct Slot {
    pub number: u64,
}

pub trait SlotProvider {
    fn run_for_every_slot<F>(&self, f: F) -> Box<dyn Future<Output = Result<()>> + Unpin + '_>
    where
        F: Fn(Slot) -> Box<dyn Future<Output = ()> + Unpin + std::marker::Send>
            + std::marker::Send
            + std::marker::Sync
            + 'static;
}

    pub async fn wait_until_slot_start(slot_number: u64) -> Result<()> {
        let slot_start_offset = GENESIS_SLOT_TIME + SLOT_PERIOD_SECONDS * slot_number;
        let slot_start = Utc.timestamp_opt(slot_start_offset as i64, 0).unwrap();
        let current_time = chrono::Utc::now();
        log::info!(
            "Current time: {}, slot {} starts: {}",
            current_time, slot_number, slot_start
        );
        if current_time < slot_start {
            let wait_time = slot_start.timestamp() - current_time.timestamp();
            log::info!(
                "Waiting for {} seconds until slot {} starts",
                wait_time,
                slot_number
            );
            tokio::time::sleep(Duration::from_secs(wait_time as u64)).await;
            Ok(())
        } else {
            log::info!("Current time is after slot start");
            Err(eyre::Error::msg(format!("Current time {} is after slot start {} for slot {}", current_time, slot_start, slot_number)))
        }
    }
