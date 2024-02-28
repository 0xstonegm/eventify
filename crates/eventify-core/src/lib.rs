#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations, unreachable_pub, rustdoc::all)]
#![deny(unused_must_use, rust_2018_idioms)]

pub mod collector;
pub mod error;
pub mod manager;
pub mod networks;
pub mod queues;

pub use collector::Collector;
pub use error::Error;
pub use manager::Manager;
pub use networks::ethereum;
pub use traits::{Collect as CollectT, Network as NetworkT};

type Result<T> = std::result::Result<T, error::Error>;

mod traits {
    use std::{fmt::Debug, future::Future};

    use alloy_primitives::B256;
    use eyre::Result;
    use jsonrpsee::core::client::{Error as RpcError, Subscription};

    use eventify_primitives::{BlockT, LogT, TransactionT};

    pub trait Collect<E>
    where
        E: std::error::Error + Send + Sync,
    {
        fn stream_blocks(&self) -> impl Future<Output = Result<(), E>>;
        fn stream_txs(&self) -> impl Future<Output = crate::Result<()>>;
        fn stream_logs(&self) -> impl Future<Output = Result<(), E>>;
    }

    pub trait Network: 'static + Clone + Debug + Send + Sync {
        type Block: BlockT;
        type LightBlock: BlockT;
        type Transaction: TransactionT;
        type Log: LogT;

        fn sub_blocks(
            &self,
        ) -> impl Future<Output = Result<Subscription<Self::LightBlock>, RpcError>> + Send;
        fn sub_txs(&self) -> impl Future<Output = Result<Subscription<B256>, RpcError>> + Send;
        fn sub_logs(
            &self,
        ) -> impl Future<Output = Result<Subscription<Self::Log>, RpcError>> + Send;
    }
}
