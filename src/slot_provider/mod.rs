//! Slot providers provide a context which takes a fn, which is then called once for every slot.
use async_trait::async_trait;

pub mod clock;
pub mod slot;

pub use slot::Slot;

#[async_trait]
pub trait SlotProvider {
    async fn get_last_slot(&self) -> Option<Slot>;
}
