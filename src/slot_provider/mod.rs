use eyre::Result;
use futures::Future;

pub mod clock;
pub mod mined_blocks;

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
