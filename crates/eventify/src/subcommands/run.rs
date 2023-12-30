use alloy_primitives::BlockNumber;
use clap::{self, Parser};
use secrecy::{ExposeSecret, Secret};

use eventify_primitives::{
    configs::{ApplicationConfig, DatabaseConfig, ServerConfig},
    Criterias,
};

#[derive(Debug, clap::Args, Clone)]
pub(crate) struct Block {
    #[arg(
        long = "src-block",
        env = "EVENTIFY_SRC_BLOCK",
        help = "The block to begin the indexing from.",
        default_value_t = 0
    )]
    src: BlockNumber,

    #[arg(
        long = "dst-block",
        env = "EVENTIFY_DST_BLOCK",
        help = "The block to end the indexing at.",
        default_value_t = BlockNumber::MAX
    )]
    dst: BlockNumber,
}

#[derive(Debug, clap::Args, Clone)]
#[group(multiple = false)]
pub(crate) struct BlockGroup {
    #[clap(flatten)]
    block: Block,

    #[arg(
        long = "from-latest",
        env = "EVENTIFY_FROM_LATEST",
        help = "Toggler enabling|disabling the indexer to run from the latest block",
        action
    )]
    latest: bool,
}

#[derive(Debug, clap::Args, Clone)]
#[group(multiple = false)]
pub(crate) struct CriteriasGroup {
    #[arg(
        long,
        env = "EVENTIFY_CRITERIAS_FILE",
        help = "file holding the criterias that'll be used to filter events",
        default_value = None,
    )]
    pub(crate) criterias_file: Option<String>,

    #[arg(
        long,
        env = "EVENTIFY_CRITERIAS_JSON",
        help = "Argument holding the criterias that'll be used to filter events",
        default_value = None,
        value_parser = clap::value_parser!(Criterias)
    )]
    pub(crate) criterias_json: Option<Criterias>,
}

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub(crate) struct Events {
    #[clap(flatten)]
    pub(crate) criterias: CriteriasGroup,
}

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub(crate) struct IndexerSettings {
    #[arg(
        long = "indexer.enabled",
        env = "EVENTIFY_INDEXER_ENABLED",
        help = "Toggler enabling|disabling the indexer",
        action
    )]
    pub(crate) indexer_enabled: bool,

    #[arg(
        long = "skip-transactions",
        env = "EVENTIFY_SKIP_TRANSACTIONS",
        help = "Toggler enabling|disabling the indexer to skip transactions",
        action
    )]
    pub(crate) skip_transactions: bool,

    #[arg(
        long = "skip-blocks",
        env = "EVENTIFY_SKIP_BLOCKS",
        help = "Toggler enabling|disabling the indexer to skip blocks",
        action
    )]
    pub(crate) skip_blocks: bool,

    #[clap(flatten)]
    pub(crate) block: BlockGroup,

    #[clap(flatten)]
    pub(crate) events: Events,
}

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub(crate) struct ServerSettings {
    #[arg(
        long = "server.enabled",
        env = "EVENTIFY_SERVER_ENABLED",
        help = "Toggler enabling|disabling the HTTP-API server",
        action
    )]
    pub(crate) server_enabled: bool,

    #[arg(
        long = "server.threads",
        env = "EVENTIFY_SERVER_THREADS",
        help = "The number of threads to use for the API server",
        default_value_t = num_cpus::get(),
    )]
    pub(crate) server_threads: usize,

    #[arg(
        long = "server.host",
        env = "EVENTIFY_SERVER_HOST",
        help = "The host to run the HTTP-API server on",
        default_value = "127.0.0.1"
    )]
    pub(crate) host: String,

    #[arg(
        long = "server.port",
        env = "EVENTIFY_SERVER_PORT",
        help = "The port to run the HTTP-API server on",
        default_value_t = 6969,
        value_parser = clap::value_parser!(u16).range(1..),
    )]
    pub(crate) port: u16,
}

#[derive(Clone, Debug, Parser)]
#[command(about = "Index Ethereum into a Postgresql database & serve it through an HTTP-API.")]
pub(crate) struct Cmd {
    #[clap(flatten)]
    pub(crate) indexer: IndexerSettings,

    #[clap(flatten)]
    pub(crate) server: ServerSettings,

    #[arg(
        long = "only-migrations",
        env = "EVENTIFY_DB_MIGRATIONS",
        help = "Run only the database migrations and exit immediately after.",
        action
    )]
    pub(crate) only_migrations: bool,

    #[arg(
        long,
        env = "DATABASE_URL",
        help = "The database URL to connect to",
        default_value = "postgres://postgres:password@localhost:5432/eventify"
    )]
    pub(crate) database_url: Secret<String>,

    #[arg(
        long,
        env = "EVENTIFY_NODE_URL",
        help = "The Ethereum node URL to connect to",
        default_value = "wss://eth.llamarpc.com"
    )]
    pub(crate) node_url: String,
    // TODO
    //#[arg(
    //    long,
    //    env = "EVENTIFY_CHAIN",
    //    help = "The chain to index",
    //    default_value_t = EVENTIFY_idx::ChainKind::Ethereum,
    //    value_parser = EVENTIFY_idx::ChainKind::from_str,
    //)]
    //pub(crate) chain: EVENTIFY_idx::ChainKind,

    //#[arg(
    //    long,
    //    env = "EVENTIFY_DATABASE",
    //    help = "The database to use",
    //    default_value_t = EVENTIFY_idx::Database::Postgres,
    //    value_parser = EVENTIFY_idx::Database::from_str,
    //)]
    //pub(crate) database: EVENTIFY_idx::Database,
}

impl From<Cmd> for ServerConfig {
    fn from(settings: Cmd) -> Self {
        Self {
            database: DatabaseConfig::from(settings.database_url()),
            application: ApplicationConfig {
                host: settings.server.host,
                port: settings.server.port,
                worker_threads: settings.server.server_threads,
            },
        }
    }
}

impl Cmd {
    #[allow(unused)]
    pub(crate) fn indexer_enabled(&self) -> bool {
        self.indexer.indexer_enabled
    }

    pub(crate) fn skip_transactions(&self) -> bool {
        self.indexer.skip_transactions
    }

    pub(crate) fn skip_blocks(&self) -> bool {
        self.indexer.skip_blocks
    }

    #[allow(unused)]
    pub(crate) fn src_block(&self) -> BlockNumber {
        if self.indexer.block.latest {
            BlockNumber::MAX
        } else {
            self.indexer.block.block.src
        }
    }

    #[allow(unused)]
    pub(crate) fn dst_block(&self) -> BlockNumber {
        if self.indexer.block.latest {
            BlockNumber::MAX
        } else {
            self.indexer.block.block.dst
        }
    }

    pub(crate) fn criterias_file(&self) -> Option<String> {
        self.indexer.events.criterias.criterias_file.clone()
    }

    pub(crate) fn criterias_json(&self) -> Option<Criterias> {
        self.indexer.events.criterias.criterias_json.clone()
    }

    #[allow(unused)]
    pub(crate) fn server_enabled(&self) -> bool {
        self.server.server_enabled
    }

    #[allow(unused)]
    pub(crate) fn server_threads(&self) -> usize {
        self.server.server_threads
    }

    #[allow(unused)]
    pub(crate) fn database_url(&self) -> &str {
        self.database_url.expose_secret()
    }

    #[allow(unused)]
    pub(crate) fn node_url(&self) -> &str {
        &self.node_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Args;
    use std::env::{remove_var, set_var};

    // as env vars are global resource and tests by default are ran in parallel
    // we need to make sure that we run them in serial mode so they don't interfere with one another
    use serial_test::serial;

    // A helper type to parse Args more easily
    #[derive(Parser)]
    struct CommandParser<T: Args> {
        #[clap(flatten)]
        args: T,
    }

    #[test]
    #[serial]
    fn test_indexer_settings_default_values() {
        let args = CommandParser::<IndexerSettings>::parse_from(["run"]).args;
        assert!(!args.indexer_enabled);
        assert!(!args.skip_transactions);
        assert!(!args.skip_blocks);
        assert!(!args.block.latest);
        assert_eq!(args.block.block.src, 0);
        assert_eq!(args.block.block.dst, BlockNumber::MAX);
        assert_eq!(args.events.criterias.criterias_file, None);
        assert_eq!(args.events.criterias.criterias_json, None);
    }

    #[test]
    #[serial]
    fn test_indexer_settings_env_values() {
        set_var("EVENTIFY_INDEXER_ENABLED", "true");
        set_var("EVENTIFY_SKIP_TRANSACTIONS", "true");
        set_var("EVENTIFY_SKIP_BLOCKS", "true");
        set_var("EVENTIFY_SRC_BLOCK", "1");
        set_var("EVENTIFY_DST_BLOCK", "2");
        set_var("EVENTIFY_CRITERIAS_FILE", "tmp/criterias.rnd");

        let args = CommandParser::<IndexerSettings>::parse_from(["run"]).args;
        assert!(args.indexer_enabled);
        assert!(args.skip_transactions);
        assert!(args.skip_blocks);
        assert_eq!(args.block.block.src, 1);
        assert_eq!(args.block.block.dst, 2);
        assert_eq!(
            args.events.criterias.criterias_file,
            Some("tmp/criterias.rnd".into())
        );
        assert_eq!(args.events.criterias.criterias_json, None);

        remove_var("EVENTIFY_INDEXER_ENABLED");
        remove_var("EVENTIFY_SKIP_TRANSACTIONS");
        remove_var("EVENTIFY_SKIP_BLOCKS");
        remove_var("EVENTIFY_SRC_BLOCK");
        remove_var("EVENTIFY_DST_BLOCK");
        remove_var("EVENTIFY_CRITERIAS_FILE");
    }

    #[test]
    #[serial]
    fn test_indexer_settings_args_precedence() {
        set_var("EVENTIFY_SRC_BLOCK", "1");
        set_var("EVENTIFY_DST_BLOCK", "2");
        set_var("EVENTIFY_CRITERIAS_JSON", "[{\"name\":\"UniswapV3Factory\",\"events\":[\"PoolCreated(address,address,uint24,int24,address)\"],\"addresses\":[\"0x1F98431c8aD98523631AE4a59f267346ea31F984\"]},{\"name\":\"ERC20\",\"events\":[\"Transfer(address,address,uint256)\",\"Approve(address,address,uint256)\"],\"addresses\":[\"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2\",\"0x6B175474E89094C44Da98b954EedeAC495271d0F\"]}]");

        let args = CommandParser::<IndexerSettings>::parse_from([
            "run",
            "--indexer.enabled",
            "--src-block",
            "3",
            "--dst-block",
            "4",
            "--criterias-json",
            "[{\"name\":\"UniswapV3Swap\",\"events\":[\"Swap(address,address,int256,int256,uint160,uint128,int24)\"],\"addresses\":[\"0x1F98431c8aD98523631AE4a59f267346ea31F984\"]}]",
        ])
        .args;
        assert!(args.indexer_enabled);
        assert!(!args.block.latest);
        assert_eq!(args.block.block.src, 3);
        assert_eq!(args.block.block.dst, 4);
        assert_eq!(
            args.events.criterias.criterias_json,
            Some("[{\"name\":\"UniswapV3Swap\",\"events\":[\"Swap(address,address,int256,int256,uint160,uint128,int24)\"],\"addresses\":[\"0x1F98431c8aD98523631AE4a59f267346ea31F984\"]}]".into())
        );
        assert_eq!(args.events.criterias.criterias_file, None);

        remove_var("EVENTIFY_SRC_BLOCK");
        remove_var("EVENTIFY_DST_BLOCK");
        remove_var("EVENTIFY_CRITERIAS_JSON");
    }

    #[test]
    #[serial]
    fn test_run_subcmd_latest() {
        let args =
            CommandParser::<Cmd>::parse_from(["run", "--indexer.enabled", "--from-latest"]).args;

        assert_eq!(args.src_block(), BlockNumber::MAX);
        assert_eq!(args.dst_block(), BlockNumber::MAX);
    }

    #[test]
    #[serial]
    fn test_server_settings_default_values() {
        let args = CommandParser::<ServerSettings>::parse_from(["run"]).args;
        assert!(!args.server_enabled);
        assert_eq!(args.server_threads, num_cpus::get());
        assert_eq!(args.host, "127.0.0.1");
        assert_eq!(args.port, 6969);
    }

    #[test]
    #[serial]
    fn test_server_settings_env_values() {
        set_var("EVENTIFY_SERVER_ENABLED", "true");
        set_var("EVENTIFY_SERVER_THREADS", "1");
        set_var("EVENTIFY_SERVER_HOST", "localhost");
        set_var("EVENTIFY_SERVER_PORT", "1234");

        let args = CommandParser::<ServerSettings>::parse_from(["run"]).args;
        assert!(args.server_enabled);
        assert_eq!(args.server_threads, 1);
        assert_eq!(args.host, "localhost");
        assert_eq!(args.port, 1234);

        remove_var("EVENTIFY_SERVER_ENABLED");
        remove_var("EVENTIFY_SERVER_THREADS");
        remove_var("EVENTIFY_SERVER_HOST");
        remove_var("EVENTIFY_SERVER_PORT");
    }

    #[test]
    #[serial]
    fn test_server_settings_args_precedence() {
        set_var("EVENTIFY_SERVER_THREADS", "1");
        set_var("EVENTIFY_SERVER_HOST", "localhost");
        set_var("EVENTIFY_SERVER_PORT", "1234");

        let args = CommandParser::<ServerSettings>::parse_from([
            "run",
            "--server.enabled",
            "--server.threads",
            "2",
            "--server.host",
            "1.2.3.4",
            "--server.port",
            "5678",
        ])
        .args;

        assert!(args.server_enabled);
        assert_eq!(args.server_threads, 2);
        assert_eq!(args.host, "1.2.3.4");

        remove_var("EVENTIFY_SERVER_THREADS");
        remove_var("EVENTIFY_SERVER_HOST");
        remove_var("EVENTIFY_SERVER_PORT");
    }
}
