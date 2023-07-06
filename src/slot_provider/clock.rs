//! Clock based SlotProvider
use async_trait::async_trait;
use chrono::Utc;
use futures::stream::StreamExt;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::IntervalStream;

use super::{Slot, SlotProvider};

const SLOT_PERIOD_SECONDS: u32 = 12;
const MAX_SLOTS: usize = 32;

/// Waits until the start of the next slot.
/// Much of our code depends on what the current slot is, and wants to answer as fast as possible,
/// therefore we sometimes want to align our code with the start of the slot.
async fn wait_until_next_slot() {
    let current_slot = Slot::now();
    let next_slot = current_slot + 1;
    let next_slot_start = next_slot.to_date_time();
    // NOTE: doesn't account for leap seconds.
    let seconds_until_next_slot = next_slot_start.timestamp() - Utc::now().timestamp();

    tokio::time::sleep(tokio::time::Duration::from_secs(
        seconds_until_next_slot as u64,
    ))
    .await;
}

pub struct SystemClockSlotProvider {
    counter: AtomicUsize,
    max_count: Option<usize>,
    slots: Mutex<VecDeque<Slot>>,
}

impl SystemClockSlotProvider {
    fn internal_new(notify: mpsc::Sender<()>, max_count: Option<usize>) -> Arc<Self> {
        let provider = Arc::new(Self {
            counter: AtomicUsize::new(0),
            max_count,
            slots: Mutex::new(VecDeque::with_capacity(MAX_SLOTS)),
        });

        log::info!("Starting slot provider with max_count {:?}", max_count);

        let provider_clone = provider.clone();
        tokio::spawn(async move {
            log::debug!("Waiting until next slot to start interval stream");
            wait_until_next_slot().await;

            // This is probably broken for leap seconds. A slot is sometimes 11s, sometimes
            // 13s long. Assuming IntervalStream waits a number of real seconds, not unix timestamp
            // seconds, we'd become misaligned.
            let mut interval_stream =
                IntervalStream::new(interval(Duration::from_secs(SLOT_PERIOD_SECONDS.into())));

            while (interval_stream.next().await).is_some() {
                let slot = Slot::now();

                log::debug!(
                    "Interval stream ticked, adding next slot to buffer {}",
                    slot
                );

                let mut slots = provider_clone.slots.lock().await;
                // When we have buffered MAX_SLOTS slots, we drop the oldest slot from the buffer.
                // Slots are picked up LIFO, the oldest slot is unlikely to ever get processed.
                // Consider adding a lifetime to a slot instead and picking a buffer which
                // can always accomodate SLOT_LIFETIME / SLOT_PERIOD_SECONDS slots.
                if slots.len() == MAX_SLOTS {
                    let oldest_slot = slots
                        .pop_front()
                        .expect("Queue to contain slots after checking length");
                    log::info!(
                        "Slots buffer full, dropping oldest slot {} from buffer",
                        oldest_slot
                    );
                }
                slots.push_back(slot);

                // Count how many slots have passed.
                provider_clone.counter.fetch_add(1, Ordering::Relaxed);

                // Notify any listener about the new slot.
                notify
                    .try_send(())
                    .unwrap_or_else(|_| println!("Error sending notification"));

                // If we are working with a max count, check if we have reached it.
                if let Some(max_count) = &provider_clone.max_count {
                    let counter = provider_clone.counter.load(Ordering::Relaxed);
                    if counter >= *max_count {
                        log::info!("Reached max count of {} slots", max_count);
                        // When we stop emitting new slots, we want to make sure any listener is notified by
                        // dropping the Sender.
                        drop(notify);
                        break;
                    }
                }
            }
        });

        provider
    }

    pub fn new(notify: mpsc::Sender<()>) -> Arc<Self> {
        Self::internal_new(notify, None)
    }

    #[cfg(test)]
    pub fn new_with_max_count(notify: mpsc::Sender<()>, max_count: usize) -> Arc<Self> {
        Self::internal_new(notify, Some(max_count))
    }
}

#[async_trait]
impl SlotProvider for SystemClockSlotProvider {
    async fn get_last_slot(&self) -> Option<Slot> {
        let slots = self.slots.lock().await;
        slots.back().cloned()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn test_wait_until_next_slot() {
        let current_slot = Slot::now();
        let next_slot = current_slot + 1;
        wait_until_next_slot().await; // Function to test
        let now = Utc::now();

        // Should be after, and within 1 second of the start of the next slot.
        assert!(now > next_slot.to_date_time());
        assert!(now - next_slot.to_date_time() < Duration::seconds(1));
    }

    #[tokio::test]
    async fn test_max_count() {
        let (tx, mut rx) = mpsc::channel(1);
        let _provider = SystemClockSlotProvider::new_with_max_count(tx, 1);

        // Wait for slot to be generated. As we wait for the start of the slot before emitting the
        // first, it takes between 0 - 12s to see the first slot appear.
        timeout(
            Duration::seconds(SLOT_PERIOD_SECONDS as i64)
                .to_std()
                .unwrap(),
            rx.recv(),
        )
        .await
        .unwrap()
        .expect("Expected one value, but no values were receieved");

        // After the last slot and notification are pushed, the provider should notice it has hit
        // its max_count, and close the notifier. This should result in a final None on the
        // notifier channel.
        match timeout(
            Duration::seconds(SLOT_PERIOD_SECONDS as i64 + 1)
                .to_std()
                .unwrap(),
            rx.recv(),
        )
        .await
        {
            Ok(last) => {
                assert!(last.is_none(), "Expected no more values to be sent")
            }
            Err(_) => panic!("Expect channel to be closed"), // Timeout was triggered as expected
        }
    }
}
