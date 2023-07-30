pub mod app;
pub mod block;
pub mod transaction;

use app::App;
use web3::types::{BlockId, BlockNumber};
use web3::Transport;

use crate::transaction::erc20::ERC20;
use crate::transaction::erc20::TRANSFER_SIGNATURE;
use crate::transaction::erc20::{self, Transfer};

type Result<T> = std::result::Result<T, crate::Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    #[error("Web3 error: {0}")]
    Web3(#[from] web3::Error),

    #[error("Migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
}

pub async fn run<T: Transport>(app: App<T>) -> Result<()> {
    let from = match app.block_from {
        BlockId::Number(block) => match block {
            BlockNumber::Number(block) => block.as_u64(),
            _ => 0,
        },
        _ => unimplemented!(),
    };

    let to = match app.block_to {
        BlockId::Number(block) => match block {
            BlockNumber::Number(block) => block.as_u64(),
            _ => app.latest_block().await?,
        },
        _ => unimplemented!(),
    };

    for target in from..=to {
        let block = app.fetch_block(BlockId::Number(target.into())).await?;
        log::debug!("Processing block {:#?}", block);

        if target % 250 == 0 {
            log::info!("Processed 250 blocks [{}, {})", target - 250, target);
        }

        let db_handler = app.dbconn().await?;
        match block {
            Some(block) => {
                let db_transaction = db_handler.begin().await?;
                block::insert(&block, &db_handler).await?;

                for tx in block.transactions {
                    // The type of transaction is determined by the initial bytes & the length of the input data
                    if tx.input.0.starts_with(TRANSFER_SIGNATURE) && tx.input.0.len() == 68 {
                        log::debug!("ERC20 transfer detected: {:#?}", tx);
                        let erc20 = ERC20::new(erc20::Method::Transfer(Transfer::from(tx)));

                        erc20.insert(&db_handler).await?;
                    }
                }

                db_transaction.commit().await?;
            }

            None => {
                log::warn!("Block {:#?} not found", block);
            }
        }
    }

    Ok(())
}
