use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::{Error, Result, WrapErr};
use futures::Future;

use crate::slot_provider::{Slot, SlotProvider};

pub struct MinedBlocksSlotProvider {
    provider: Provider<Ws>,
}

impl MinedBlocksSlotProvider {
    pub async fn new() -> Result<MinedBlocksSlotProvider> {
        let provider = Provider::<Ws>::connect(
            "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .await?;
        Ok(MinedBlocksSlotProvider { provider })
    }
}

impl SlotProvider for MinedBlocksSlotProvider {
    fn run_for_every_slot<F>(&self, f: F) -> Box<dyn Future<Output = Result<()>> + Unpin + '_>
    where
        F: Fn(Slot) -> Result<()> + 'static,
    {
        Box::new(Box::pin(async move {
            let block_stream = self.provider.subscribe_blocks().await?;
            let mut slot_stream = block_stream.map(|block| -> Result<Slot>{ Ok(Slot {
                number: block.number.ok_or(Error::msg("block.number is none"))?.as_u64(),
            })});
            while let Some(slot_result) = slot_stream.next().await {
                let slot = match slot_result {
                    Ok(slot) => slot,
                    Err(e) => {
                        log::error!("Error getting slot: {:?}", e);
                        continue;
                    }
                };
                let slot_number = slot.number;
                match f(slot).wrap_err(format!("Failed to run for slot: {}", slot_number)) {
                    Ok(_) => log::info!("Ran succesfully for slot {}", slot_number),
                    Err(e) => log::error!("{:?}", e),
                };
            }
            Ok(())
        }))
    }
}
