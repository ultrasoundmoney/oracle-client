use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::{Result, WrapErr};
use futures::Future;
use tokio::time::{interval, Duration};
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};

use crate::slot_provider::{Slot, SlotProvider};
use tokio_stream::wrappers::IntervalStream;

pub struct SystemClockSlotProvider {
    num_slots: Option<usize>,
}

impl SystemClockSlotProvider {
    pub fn new() -> Self {
        Self { num_slots: None }
    }

    pub fn stop_after_num_slots(num_slots: usize) -> Self {
        Self {
            num_slots: Some(num_slots),
        }
    }

    async fn wait_until_slot_start(slot_number: u64) -> Result<()> {
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
}

// Dec-01-2020 12:00:23 UTC
pub const GENESIS_SLOT_TIME: u64 = 1606824023;
pub const SLOT_PERIOD_SECONDS: u64 = 12;
const MAX_CONCURRENT_SLOTS: usize = 8;

impl SlotProvider for SystemClockSlotProvider {
    fn run_for_every_slot<F>(&self, f: F) -> Box<dyn Future<Output = Result<()>> + Unpin + '_>
    where
        F: Fn(Slot) -> Box<dyn Future<Output = ()> + Unpin + std::marker::Send>
            + std::marker::Send
            + std::marker::Sync
            + 'static,
    {
        Box::new(Box::pin(async move {
            let slot_stream =
                IntervalStream::new(interval(Duration::from_secs(SLOT_PERIOD_SECONDS))).map(|_| {
                    let now = chrono::Utc::now().timestamp();
                    let slot_number = (now - GENESIS_SLOT_TIME as i64) / SLOT_PERIOD_SECONDS as i64 + 1;
                    Slot {
                        number: slot_number as u64,
                    }
                });


            if let Some(num_slots) = self.num_slots {
                log::info!("Stopping after {} slots", num_slots);
                slot_stream
                    .take(num_slots)
                    .for_each_concurrent(MAX_CONCURRENT_SLOTS, |slot| async {
                        let slot_number = slot.number;
                        let wait_result = SystemClockSlotProvider::wait_until_slot_start(slot_number)
                            .await;
                        match wait_result {
                            Ok(_) => {}
                            Err(_) => {
                                log::error!(
                                    "Error waiting for slot {}: {:?}",
                                    slot.number,
                                    wait_result
                                );
                                return;
                            }
                        }
                        tokio::spawn(f(slot)).await.unwrap_or_else(|e| {
                            log::error!("Error spawning task for slot {}: {:?}", slot_number, e);
                        })
                    })
                    .await;
            } else {
                slot_stream
                    .for_each_concurrent(MAX_CONCURRENT_SLOTS, |slot| async {
                        let slot_number = slot.number;
                        let wait_result = SystemClockSlotProvider::wait_until_slot_start(slot_number)
                            .await;
                        match wait_result {
                            Ok(_) => {}
                            Err(_) => {
                                log::error!(
                                    "Error waiting for slot {}: {:?}",
                                    slot.number,
                                    wait_result
                                );
                                return;
                            }
                        }
                        tokio::spawn(f(slot)).await.unwrap_or_else(|e| {
                            log::error!("Error spawning task for slot {}: {:?}", slot_number, e);
                        })
                    })
                    .await;
            }
            Ok(())
        }))
    }
}
