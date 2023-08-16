#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid transaction input length {0}")]
    InvalidTransactionInputLength(usize),

    #[error("Invalid transaction function signature {0}")]
    InvalidTransactionFunctionSignature(String),

    #[error("Unindexed transaction")]
    UnindexedTransaction,

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
