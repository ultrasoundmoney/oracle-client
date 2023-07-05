use std::{fmt::Display, ops::Add};

use chrono::{DateTime, Duration, Utc};
use eyre::Result;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BEACON_GENESIS: DateTime<Utc> = "2020-12-01T12:00:23Z".parse().unwrap();
}

/// A slot number on the beacon chain.
/// Started at 0 at 2020-12-01T12:00:23Z (genesis). Slots follow unix timestamps meaning most slots
/// are 12 seconds long, but some are 13 or 11 seconds long. We use u64 to store. Enough for 12
/// seconds * 2^64 = ~7.02e12 years. We should probably use u32 instead.
#[derive(Clone, Copy, Debug)]
pub struct Slot(pub u64);

impl Slot {
    // Note: this is true virtually all of the time but because of leap seconds not always.
    pub const SLOT_PERIOD_SECONDS: u64 = 12;

    /// Create a slot from a timestamp.
    /// Rounds down to the nearest slot. i.e. what slot was the timestamp in?
    pub fn from_date_time_round_down(date_time: DateTime<Utc>) -> Result<Slot> {
        if date_time < *BEACON_GENESIS {
            eyre::bail!(
                "cannot convert DateTime ({}) before beacon genesis ({}) to Slot",
                date_time,
                *BEACON_GENESIS
            )
        }

        let slot = Slot(
            ((date_time - *BEACON_GENESIS).num_seconds() / Self::SLOT_PERIOD_SECONDS as i64) as u64,
        );
        Ok(slot)
    }

    pub fn to_date_time(self) -> DateTime<Utc> {
        *BEACON_GENESIS + Duration::seconds(self.0 as i64 * Self::SLOT_PERIOD_SECONDS as i64)
    }

    pub fn now() -> Self {
        Self::from_date_time_round_down(Utc::now()).expect("Expect now to be after beacon genesis")
    }
}

impl From<Slot> for DateTime<Utc> {
    fn from(slot: Slot) -> Self {
        *BEACON_GENESIS + Duration::seconds(slot.0 as i64 * Slot::SLOT_PERIOD_SECONDS as i64)
    }
}

impl Add<u64> for Slot {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Slot(self.0 + rhs)
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>7}", self.0)
    }
}
