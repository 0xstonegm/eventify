use std::str::FromStr;

use clap::Parser;

use eventify_idx::ChainKind;

// TODO: implement the stream subcmd
#[derive(Clone, Debug, Parser)]
#[command(about = "[NOT YET IMPLEMENTED] Subscribe & stream directly from the tip of the chain")]
pub(crate) struct Cmd {
    #[arg(
        long,
        env = "EVENTIFY_CHAIN",
        help = "The type of chain to index",
        default_value_t = ChainKind::Ethereum,
        value_parser = ChainKind::from_str,
    )]
    pub(crate) chain: ChainKind,

    #[arg(
        long,
        env = "EVENTIFY_NODE_URL",
        help = "The node URL to connect to",
        default_value = "wss://eth.llamarpc.com"
    )]
    pub(crate) node_url: String,
}
