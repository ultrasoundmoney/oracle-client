use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::{Result, WrapErr};
use futures::Future;
use tokio::time::{interval, Duration};

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
}

// Dec-01-2020 12:00:23 UTC
const GENESIS_SLOT_TIME: u64 = 1606824023;
const SLOT_PERIOD_SECONDS: u64 = 12;

impl SlotProvider for SystemClockSlotProvider {
    fn run_for_every_slot<F>(&self, f: F) -> Box<dyn Future<Output = Result<()>> + Unpin + '_>
    where
        F: Fn(Slot) -> Box<dyn Future<Output = ()> + Unpin + std::marker::Send>
            + std::marker::Send
            + std::marker::Sync
            + 'static,
    {
        Box::new(Box::pin(async move {
            let mut slot_stream =
                IntervalStream::new(interval(Duration::from_secs(SLOT_PERIOD_SECONDS))).map(|_| {
                    let now = chrono::Utc::now().timestamp();
                    let slot_number = (now - GENESIS_SLOT_TIME as i64) / SLOT_PERIOD_SECONDS as i64;
                    Slot {
                        number: slot_number as u64,
                    }
                });

            let next_slot_start = GENESIS_SLOT_TIME
                + SLOT_PERIOD_SECONDS * (slot_stream.next().await.unwrap().number + 1);

            // Wait until the next slot starts
            let now = chrono::Utc::now().timestamp();
            let wait_time = next_slot_start - now as u64;
            log::info!(
                "Waiting for {} seconds until the first slot starts",
                wait_time
            );
            tokio::time::sleep(Duration::from_secs(wait_time)).await;

            if let Some(num_slots) = self.num_slots {
                log::info!("Stopping after {} slots", num_slots);
                slot_stream
                    .take(num_slots)
                    .for_each_concurrent(4, |slot| async {
                        tokio::spawn(f(slot)).await;
                    })
                    .await;
            } else {
                slot_stream
                    .for_each_concurrent(4, |slot| async {
                        tokio::spawn(f(slot)).await;
                    })
                    .await;
            }
            Ok(())
        }))
    }
}
