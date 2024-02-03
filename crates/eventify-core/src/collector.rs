use std::time::Instant;

use alloy_primitives::{BlockNumber, FixedBytes};

use tokio::sync::watch::Receiver;
use tracing::{info, trace};

use crate::{emit::Emit, provider::Node, Collect, Store};
use eventify_configs::core::CollectorConfig;
use eventify_primitives::{consts, Criteria, LogKind, ResourceKind};

#[derive(Debug, Clone)]
pub struct Collector<N, S, E>
where
    N: Node,
    S: Store,
    E: Emit,
{
    config: CollectorConfig,
    node: N,
    storage: S,
    mid: E,
}

impl<N, S, E> Collector<N, S, E>
where
    N: Node,
    S: Store,
    E: Emit,
{
    pub fn new(config: CollectorConfig, node: N, storage: S, mid: E) -> Self {
        Self {
            config,
            node,
            storage,
            mid,
        }
    }

    pub async fn get_latest_block(&self) -> crate::Result<BlockNumber> {
        self.node.get_block_number().await.map_err(Into::into)
    }
}

impl<N, S, E> Collect<crate::Error> for Collector<N, S, E>
where
    N: Node,
    S: Store,
    E: Emit,
{
    async fn collect_block(&self, block: BlockNumber) -> crate::Result<()> {
        let block = self.node.get_block(block).await?;
        self.storage.store_block(&block).await?;
        self.mid
            .publish(&self.config.network, &ResourceKind::Block, &block)?;
        Ok(())
    }

    async fn collect_blocks(
        &self,
        signal_receiver: Receiver<bool>,
        from: BlockNumber,
        to: BlockNumber,
    ) -> crate::Result<()> {
        info!(from_block=?from, to_block=?to);
        let now = Instant::now();

        for block in from..=to {
            if signal_receiver.borrow().to_owned() {
                trace!("Received a signal to stop processing blocks");
                break;
            };

            self.collect_block(block).await?;
            if block % 30 == 0 {
                info!(
                    processed=?true, block_count=?block - from,
                    latest=?block, elapsed=?now.elapsed());
            }
        }

        Ok(())
    }

    async fn collect_transactions(&self, block: BlockNumber) -> crate::Result<()> {
        let now = Instant::now();
        let tx = self.node.get_transactions(block).await?;
        let tx_count = tx.len();

        for tx in tx {
            self.storage.store_transaction(&tx).await?;
            self.mid
                .publish(&self.config.network, &ResourceKind::Transaction, &tx)?;
        }
        info!(processed=?true, tx_count=?tx_count, block=?block, elapsed=?now.elapsed());

        Ok(())
    }

    async fn collect_transactions_from_range(
        &self,
        signal_receiver: Receiver<bool>,
        from: BlockNumber,
        to: BlockNumber,
    ) -> crate::Result<()> {
        info!("Processing transactions from blocks {}..{}", from, to);

        for block in from..=to {
            if signal_receiver.borrow().to_owned() {
                trace!("Received a signal to stop processing transactions");
                break;
            };

            self.collect_transactions(block).await?;
        }

        Ok(())
    }

    async fn collect_logs(
        &self,
        signal_receiver: Receiver<bool>,
        criteria: &Criteria,
    ) -> crate::Result<()> {
        let now = Instant::now();

        let logs = self.node.get_logs(criteria).await?;
        let mut log_count = 0;

        for log in logs {
            if signal_receiver.borrow().to_owned() {
                trace!("Received a signal to stop processing logs");
                break;
            };

            log_count += 1;
            self.storage.store_log(&log).await?;
            self.mid
                .publish(&self.config.network, &ResourceKind::Log(LogKind::Raw), &log)?;

            // TODO add an additional thread that'll keep track of the resources' current stats
            if log_count % 100 == 0 {
                info!(processed=?true, log_count=?log_count, latest_tx_hash=?log.transaction_hash, elapsed=?now.elapsed());
            }
        }

        Ok(())
    }

    async fn stream_blocks(&self, signal_receiver: Receiver<bool>) -> crate::Result<()> {
        let mut stream = self.node.stream_blocks().await?;

        while let Some(block) = stream.next().await {
            if signal_receiver.borrow().to_owned() {
                trace!("Received a signal to stop streaming blocks");
                break;
            };
            let block = block?;
            trace!(block=?block);
            info!(kind="block", number=?block.number, hash=?block.hash);
            self.storage.store_block(&block).await?;
            self.mid
                .publish(&self.config.network, &ResourceKind::Block, &block)?;
        }

        Ok(())
    }

    async fn stream_transactions(&self, signal_receiver: Receiver<bool>) -> crate::Result<()> {
        let mut stream = self.node.stream_blocks().await?;

        while let Some(block) = stream.next().await {
            if signal_receiver.borrow().to_owned() {
                trace!("Received a signal to stop streaming transactions");
                break;
            };

            let block = block?;
            let tx = self
                .node
                .get_transactions(block.number.expect("Invalid block number").to::<u64>())
                .await?;
            for tx in tx {
                trace!(tx=?tx);
                info!(kind="tx", hash=?tx.hash);
                self.storage.store_transaction(&tx).await?;
                self.mid
                    .publish(&self.config.network, &ResourceKind::Transaction, &tx)?;
            }
        }

        Ok(())
    }

    async fn stream_logs(&self, signal_receiver: Receiver<bool>) -> crate::Result<()> {
        let mut stream = self.node.stream_logs().await?;

        while let Some(log) = stream.next().await {
            if signal_receiver.borrow().to_owned() {
                trace!("Received a signal to stop streaming logs");
                break;
            };

            let log = log?;
            trace!(log=?log);
            match log.topics.first() {
                Some(topic) => {
                    if topic == consts::TRANSFER.parse::<FixedBytes<32>>().as_ref().unwrap() {
                        info!(kind=LogKind::Transfer.to_string(), address=?log.address, tx_hash=?log.transaction_hash);
                        self.storage
                            .store_log_transfer(
                                &log.transaction_hash.unwrap_or_default(),
                                log.topics.get(1).unwrap_or_default(),
                                log.topics.get(2).unwrap_or_default(),
                                log.data.clone(),
                            )
                            .await?;
                        self.mid.publish(
                            &self.config.network,
                            &ResourceKind::Log(LogKind::Transfer),
                            &log,
                        )?;
                    } else if topic == consts::APPROVAL.parse::<FixedBytes<32>>().as_ref().unwrap()
                    {
                        info!(kind=LogKind::Approval.to_string(), address=?log.address, tx_hash=?log.transaction_hash);
                        self.storage
                            .store_log_approval(
                                &log.transaction_hash.unwrap_or_default(),
                                log.topics.get(1).unwrap_or_default(),
                                log.topics.get(2).unwrap_or_default(),
                                log.data.clone(),
                            )
                            .await?;
                        self.mid.publish(
                            &self.config.network,
                            &ResourceKind::Log(LogKind::Approval),
                            &log,
                        )?;
                    } else if topic
                        == consts::APPROVAL_FOR_ALL
                            .parse::<FixedBytes<32>>()
                            .as_ref()
                            .unwrap()
                    {
                        info!(kind=LogKind::ApprovalForAll.to_string(), address=?log.address, tx_hash=?log.transaction_hash);
                        self.storage
                            .store_log_approval_for_all(
                                &log.transaction_hash.unwrap_or_default(),
                                log.topics.get(1).unwrap_or_default(),
                                log.topics.get(2).unwrap_or_default(),
                                log.data.ends_with(&[0x1]),
                            )
                            .await?;
                        self.mid.publish(
                            &self.config.network,
                            &ResourceKind::Log(LogKind::ApprovalForAll),
                            &log,
                        )?;
                    } else {
                        info!(kind=LogKind::Raw.to_string(), address=?log.address, tx_hash=?log.transaction_hash);
                        self.storage.store_log(&log).await?;
                        self.mid.publish(
                            &self.config.network,
                            &ResourceKind::Log(LogKind::Raw),
                            &log,
                        )?;
                    }
                }
                None => {
                    info!(kind=LogKind::Raw.to_string(), address=?log.address, tx_hash=?log.transaction_hash);
                    self.storage.store_log(&log).await?;
                    self.mid.publish(
                        &self.config.network,
                        &ResourceKind::Log(LogKind::Raw),
                        &log,
                    )?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{hex::FromHex, Address, Bytes};

    #[test]
    fn test_bytes() {
        let b =
            Bytes::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        println!("{:?}", &b[15..]);
        assert!(b.ends_with(&[0x1]));

        let b =
            Bytes::from_hex("0x0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        assert!(b.ends_with(&[0x0]));

        let sent = Bytes::from_hex("0x0000000000000000000000000000000000000000000009976cd8feec903400000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        println!("{:?}", Address::try_from(&sent[12..32]).unwrap_or_default());
    }
}
