pub mod ethereum;

use std::sync::Arc;

use reconnecting_jsonrpsee_ws_client::{Client, RpcError};

#[derive(Clone)]
pub struct NetworkClient {
    inner: Arc<Client>,
    pub host: String,
}

impl NetworkClient {
    pub async fn new(host: String) -> Result<NetworkClient, RpcError> {
        Ok(Self {
            inner: std::sync::Arc::new(
                reconnecting_jsonrpsee_ws_client::Client::builder()
                    .retry_policy(reconnecting_jsonrpsee_ws_client::FixedInterval::from_millis(500))
                    .enable_ws_ping(
                        reconnecting_jsonrpsee_ws_client::PingConfig::new()
                            .ping_interval(std::time::Duration::from_secs(6))
                            .inactive_limit(std::time::Duration::from_secs(30)),
                    )
                    .build(host.clone())
                    .await?,
            ),
            host,
        })
    }
}

impl std::fmt::Display for NetworkClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", stringify!(NetworkClient))
    }
}

impl std::fmt::Debug for NetworkClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", stringify!(NetworkClient))
    }
}
