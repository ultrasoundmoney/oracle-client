//! Slot providers provide a context which takes a fn, which is then called once for every slot.
//! TODO: convert timestamp to DateTime
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use eyre::{Error, Result};
use lazy_static::lazy_static;

pub mod clock;

lazy_static! {
    pub static ref BEACON_GENESIS: DateTime<Utc> = "2020-12-01T12:00:23Z".parse().unwrap();
}

// Dec-01-2020 12:00:23 UTC
pub const GENESIS_SLOT_TIME: u64 = 1606824023;
pub const SLOT_PERIOD_SECONDS: u64 = 12;

// TODO: convert to unit struct.
// TODO: see if Copy is needed after unit struct.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub number: u64,
}

impl Slot {
    /// Create a slot from a timestamp.
    /// Rounds down to the nearest slot. i.e. what slot was the timestamp in?
    pub fn from_timestamp(timestamp: u64) -> Result<Slot> {
        if timestamp < GENESIS_SLOT_TIME {
            Err(Error::msg(format!(
                "Timestamp {} is before genesis slot time {}",
                timestamp, GENESIS_SLOT_TIME
            )))
        } else {
            let slot = Slot {
                number: (timestamp - GENESIS_SLOT_TIME) / SLOT_PERIOD_SECONDS,
            };
            Ok(slot)
        }
    }

    pub fn to_date_time(self) -> DateTime<Utc> {
        *BEACON_GENESIS + Duration::seconds(self.number as i64 * SLOT_PERIOD_SECONDS as i64)
    }
}

#[async_trait]
pub trait SlotProvider {
    async fn get_last_slot(&self) -> Option<Slot>;
}
