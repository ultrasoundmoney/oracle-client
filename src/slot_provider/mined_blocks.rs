use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::{Error, Result, WrapErr};
use futures::Future;

use crate::slot_provider::{Slot, SlotProvider};

pub struct MinedBlocksSlotProvider {
    provider: Provider<Ws>,
    num_blocks: usize,
}

impl MinedBlocksSlotProvider {
    pub async fn new(num_blocks: Option<usize>) -> Result<MinedBlocksSlotProvider> {
        let provider = Provider::<Ws>::connect(
            "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .await?;
        Ok(MinedBlocksSlotProvider {
            provider,
            num_blocks: if num_blocks.is_some() {
                num_blocks.unwrap()
            } else {
                usize::MAX
            },
        })
    }
}

impl SlotProvider for MinedBlocksSlotProvider {
    fn run_for_every_slot<'a, F>(&'a self, f: F) -> Box<dyn Future<Output = Result<()>> + Unpin + '_>
    where
        F: Fn(Slot) -> Box<dyn Future<Output = Result<()>> + Unpin> + 'a
    {
        Box::new(Box::pin(async move {
            let block_stream = self.provider.subscribe_blocks().await?;

            let mut slot_stream = block_stream
                .take(self.num_blocks)
                .map(|block| -> Result<Slot> {
                    Ok(Slot {
                        number: block
                            .number
                            .ok_or(Error::msg("block.number is none"))?
                            .as_u64(),
                    })
                });

            while let Some(slot_result) = slot_stream.next().await {
                let slot = match slot_result {
                    Ok(slot) => slot,
                    Err(e) => {
                        log::error!("Error getting slot: {:?}", e);
                        continue;
                    }
                };
                let slot_number = slot.number;
                match f(slot).await.wrap_err(format!("Failed to run for slot: {}", slot_number)) {
                    Ok(_) => log::info!("Ran succesfully for slot {}", slot_number),
                    Err(e) => log::error!("{:?}", e),
                };
            }
            Ok(())
        }))
    }
}
