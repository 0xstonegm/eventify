use tokio::{
    sync::{mpsc, watch::Receiver},
    task::JoinHandle,
};
use tracing::{error, info, warn};

use crate::{
    collector::Collector,
    traits::{Collect, Network},
};
use eventify_configs::configs::{CollectorConfig, ManagerConfig};
use eventify_primitives::networks::{Resource, ResourceKind};

#[derive(Debug, Clone)]
pub struct Manager<N>
where
    N: Network,
{
    pub manager_config: ManagerConfig,
    pub collector_config: CollectorConfig,
    queue_rx: mpsc::Sender<Resource<N::Block, N::Log>>,
}

impl<N> Manager<N>
where
    N: Network,
{
    pub fn new(
        manager_config: ManagerConfig,
        collector_config: CollectorConfig,
        queue_rx: mpsc::Sender<Resource<N::Block, N::Log>>,
    ) -> Self {
        Self {
            manager_config,
            collector_config,
            queue_rx,
        }
    }
}

impl<N> Manager<N>
where
    N: Network,
{
    pub async fn init_stream_tasks(
        &self,
        stop_signal: Receiver<bool>,
    ) -> crate::Result<Vec<JoinHandle<()>>> {
        let mut tasks = Vec::new();

        for r in self.manager_config.resources.iter() {
            tasks.push(
                self.create_stream_task(r, &self.collector_config, &stop_signal)
                    .await?,
            );
        }

        Ok(tasks)
    }

    async fn create_stream_task(
        &self,
        resource: &ResourceKind,
        collector_config: &CollectorConfig,
        stop_signal: &Receiver<bool>,
    ) -> crate::Result<JoinHandle<()>> {
        let collector: Collector<N> =
            Collector::new(collector_config.clone(), self.queue_rx.clone()).await?;
        let resource = *resource;
        let stop_signal = stop_signal.clone();

        Ok(tokio::spawn(async move {
            let stream_result = match resource {
                ResourceKind::Block => collector.stream_blocks(stop_signal).await,
                ResourceKind::Log(_) => collector.stream_logs(stop_signal).await,
            };

            match stream_result {
                Ok(()) => {
                    info!(thread=?resource, "Finished streaming");
                }
                Err(err) => match err {
                    crate::Error::EmptyStream => {
                        warn!(err = "Empty stream");
                    }
                    crate::Error::SignalRecv(err) => {
                        info!(err = "Received stop signal", ?err);
                    }
                    _ => {
                        error!(err=?err);
                    }
                },
            }
        }))
    }
}
