use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::Result;
use futures::Future;

use crate::slot_provider::{Slot, SlotProvider};

pub struct MinedBlocksSlotProvider {
    provider: Provider<Ws>,
}

impl MinedBlocksSlotProvider {
    pub async fn new() -> MinedBlocksSlotProvider {
        let provider = Provider::<Ws>::connect(
            "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .await
        .expect("Error connecting to Infura");
        MinedBlocksSlotProvider { provider }
    }
}

impl SlotProvider for MinedBlocksSlotProvider {
    fn run_for_every_slot<F>(&self, f: F) -> Box<dyn Future<Output = Result<()>> + Unpin + '_>
    where
        F: Fn(Slot) + 'static,
    {
        Box::new(Box::pin(async move {
            let block_stream = self.provider.subscribe_blocks().await?;
            let mut slot_stream = block_stream.map(|block| Slot {
                number: block.number.unwrap().as_u64(),
            });
            while let Some(slot) = slot_stream.next().await {
                f(slot);
            }
            Ok(())
        }))
    }
}
