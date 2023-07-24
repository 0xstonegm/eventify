use sqlx::postgres::PgPool;
use web3::transports::{ipc::Ipc, ws::WebSocket, Http};
use web3::types::{Block, BlockId, Transaction};
use web3::{Transport, Web3};

use crate::{block, Result};

#[derive(Debug)]
struct App<T: Transport> {
    block_from: BlockId,
    block_to: BlockId,

    inner: Inner<T>,
}

impl<T: Transport> App<T> {
    /// Create a new instance of the indexer
    pub fn new(
        block_from: BlockId,
        block_to: BlockId,
        transport_node: Option<Web3<T>>,
        transport_db: Option<PgPool>,
    ) -> Self {
        Self {
            inner: Inner::new(transport_node, transport_db),
            block_from,
            block_to,
        }
    }

    pub fn with_from_block(self, block_from: BlockId) -> Self {
        Self { block_from, ..self }
    }

    pub fn with_to_block(self, block_to: BlockId) -> Self {
        Self { block_to, ..self }
    }

    pub fn with_node_conn(self, transport: Web3<T>) -> Self {
        Self {
            inner: Inner::new(Some(transport), self.inner.transport_db),
            ..self
        }
    }

    pub fn with_database_conn(self, database_conn: PgPool) -> Self {
        Self {
            inner: Inner::new(self.inner.transport_node, Some(database_conn)),
            ..self
        }
    }

    pub async fn with_database_url(self, database_url: &str) -> Self {
        Self {
            inner: Inner::new(
                self.inner.transport_node,
                Some(
                    PgPool::connect(database_url)
                        .await
                        .expect("Failed to connect to the database with the provided URL"),
                ),
            ),
            ..self
        }
    }
}

/*
pub async fn run(&self) -> std::result::Result<(), crate::Error> {
    let db_conn = sqlx::PgPool::connect(&self.inner.database_url).await?;
    sqlx::migrate!().run(&db_conn).await?;

    let conn = web3::Web3::new(web3::transports::Http::new(&self.inner.node_url)?);

    let begin = self.inner.from_block;
    let end = match self.inner.to_block {
        Some(block) => block,
        None => conn.eth().block_number().await?.as_u64(),
    };

    for block in begin..=end {
        // Retrieve the block with transactions
        let block_with_txs = conn
            .eth()
            .block_with_txs(BlockId::Number(block.into()))
            .await?;

        if let Some(block) = block_with_txs {
            insert_block(&block, &db_conn).await?;

            for tx in block.transactions {
                log::info!("{:?}", tx);
                if tx.input.0.starts_with(TRANSFER_SIGNATURE) && tx.input.0.len() == 68 {
                    let transfer = erc20::Method::Transfer(erc20::transfer::Transfer {
                        hash: tx.hash,
                        from: tx.from.unwrap(),
                        to: H160::from_slice(&tx.input.0[16..36]),
                        value: U256::from(&tx.input.0[36..68]),
                    });

                    log::info!("{:?}", transfer);
                    if let Some(to) = tx.to {
                        let erc20 = erc20::ERC20::new(to, transfer);
                        erc20.insert(&db_conn).await?;
                    }
                }
            }
        }
    }
    Ok(())
}
*/

impl App<Http> {
    /// Create a new instance of the indexer with the HTTP transport
    #[allow(unused)]
    pub fn with_http(self, node_url: String) -> Self {
        Self {
            inner: Inner::new(
                Some(Web3::new(
                    Http::new(&node_url).expect("Failed to create HTTP transport"),
                )),
                self.inner.transport_db,
            ),
            ..self
        }
    }

    pub async fn run(&self) -> Result<Option<Block<Transaction>>> {
        let block = self
            .inner
            .transport_node
            .as_ref()
            .expect("No transport node provided")
            .clone()
            .eth()
            .block_with_txs(self.block_from)
            .await?;

        Ok(block)
    }
}

impl App<Ipc> {
    /// Create a new instance of the indexer with the IPC transport
    #[allow(unused)]
    pub async fn with_ipc(self, node_url: String) -> Self {
        Self {
            inner: Inner::new(
                Some(Web3::new(
                    Ipc::new(&node_url)
                        .await
                        .expect("Failed to create HTTP transport"),
                )),
                self.inner.transport_db,
            ),
            ..self
        }
    }

    pub async fn run(&self) -> Result<Option<Block<Transaction>>> {
        let block = self
            .inner
            .transport_node
            .as_ref()
            .expect("No transport node provided")
            .clone()
            .eth()
            .block_with_txs(self.block_from)
            .await?;

        Ok(block)
    }
}

impl App<WebSocket> {
    /// Create a new instance of the indexer with the WebSocket transport
    #[allow(unused)]
    pub async fn with_websocket(self, node_url: String) -> Self {
        Self {
            inner: Inner::new(
                Some(Web3::new(
                    WebSocket::new(&node_url)
                        .await
                        .expect("Failed to create HTTP transport"),
                )),
                self.inner.transport_db,
            ),
            ..self
        }
    }

    /// Create a new instance of the indexer with the WebSocket transport
    ///
    /// An alias for `with_websocket`
    pub async fn with_ws(self, node_url: String) -> Self {
        self.with_websocket(node_url).await
    }

    pub async fn run(&self) -> Result<Option<Block<Transaction>>> {
        let block: Option<Block<Transaction>> = self
            .inner
            .transport_node
            .as_ref()
            .expect("No transport node provided")
            .clone()
            .eth()
            .block_with_txs(self.block_from)
            .await?;

        Ok(block)
    }
}

#[derive(Debug)]
struct Inner<T: Transport> {
    transport_node: Option<Web3<T>>,
    transport_db: Option<PgPool>,
}

impl<T: Transport> Default for Inner<T> {
    fn default() -> Self {
        Self {
            transport_node: None,
            transport_db: None,
        }
    }
}

impl<T: Transport> Inner<T> {
    pub fn new(node: Option<Web3<T>>, db: Option<PgPool>) -> Self {
        Self {
            transport_node: node,
            transport_db: db,
        }
    }
}
