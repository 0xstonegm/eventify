#![doc = include_str!("../README.md")]

pub mod app;
pub mod collector;
pub mod error;
pub mod manager;
pub mod types;

pub use app::App;
pub use collector::Collector;
pub use error::Error;
pub use manager::Manager;
pub use types::{process::Process, runner::Runner};

/// The Result used throughout the indexer
type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug)]
pub enum SupportedChains {
    Ethereum,
}
