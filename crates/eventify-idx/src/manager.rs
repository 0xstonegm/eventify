use ethers_providers::JsonRpcClient;

use crate::{types::provider::NodeProvider, Collector, Process, Result, Runner};
use eventify_primitives::{Auth, IndexedBlock, IndexedLog, Storage};

#[derive(Debug, Clone, Default)]
pub struct Manager;

impl Manager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Runner for Manager {
    type Error = crate::Error;

    async fn run<T: NodeProvider + JsonRpcClient + 'static, U: Storage>(
        processor: Collector<T, U>,
    ) -> std::result::Result<(), Self::Error> {
        processor.process_all_serial().await?;

        Ok(())
    }

    async fn run_par<T: NodeProvider + JsonRpcClient + 'static, U: Storage>(
        processor: Collector<T, U>,
    ) -> Result<()> {
        let block_processor = processor.clone();
        let log_processor = processor.clone();

        let handles = vec![
            tokio::spawn(async move {
                <Collector<T, U> as Process<IndexedLog>>::process(&log_processor).await
            }),
            tokio::spawn(async move {
                <Collector<T, U> as Process<IndexedBlock>>::process(&block_processor).await
            }),
        ];

        futures::future::join_all(handles).await;

        Ok(())
    }
}
