use ethers::providers::StreamExt;
use eyre::Result;
use futures::Future;
use tokio::time::{interval, Duration};

use crate::slot_provider::{
    wait_until_slot_start, Slot, SlotProvider, GENESIS_SLOT_TIME, SLOT_PERIOD_SECONDS,
};
use tokio_stream::wrappers::IntervalStream;

pub struct SystemClockSlotProvider {
    num_slots: Option<usize>,
}

impl SystemClockSlotProvider {
    pub fn new() -> Self {
        Self { num_slots: None }
    }

    #[allow(dead_code)]
    pub fn stop_after_num_slots(num_slots: usize) -> Self {
        Self {
            num_slots: Some(num_slots),
        }
    }
}

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
                    let slot_number =
                        (now - GENESIS_SLOT_TIME as i64) / SLOT_PERIOD_SECONDS as i64 + 1;
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
                        // NOTE: I previously had moved this waiting into the handler function f
                        // which resulted in the interval stream not triggering correctly anymore
                        wait_until_slot_start(slot_number)
                            .await
                            .unwrap_or_else(|e| {
                                log::error!("Error waiting for slot {}: {:?}", slot_number, e);
                            });
                        tokio::spawn(f(slot)).await.unwrap_or_else(|e| {
                            log::error!("Error spawning task for slot {}: {:?}", slot_number, e);
                        })
                    })
                    .await;
            } else {
                slot_stream
                    .for_each_concurrent(MAX_CONCURRENT_SLOTS, |slot| async {
                        let slot_number = slot.number;
                        wait_until_slot_start(slot_number)
                            .await
                            .unwrap_or_else(|e| {
                                log::error!("Error waiting for slot {}: {:?}", slot_number, e);
                            });
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
