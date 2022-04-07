use clap::{Parser, Subcommand};
use feth::BLOCK_TIME;
use std::path::PathBuf;
use web3::types::{Address, H256};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub(crate) struct Cli {
    /// The maximum parallelism
    #[clap(long, default_value_t = 200)]
    pub(crate) max_parallelism: u64,

    /// The count of transactions sent by a routine
    #[clap(long, default_value_t = 0)]
    pub(crate) count: u64,

    /// the source account file
    #[clap(long, parse(from_os_str), value_name = "FILE", default_value = "source_keys.001")]
    pub(crate) source: PathBuf,

    /// block time of the network
    #[clap(long, default_value_t = BLOCK_TIME)]
    pub(crate) block_time: u64,

    /// findora network full-node urls: http://node0:8545,http://node1:8545
    #[clap(long)]
    pub(crate) network: Option<String>,

    /// http request timeout, seconds
    #[clap(long)]
    pub(crate) timeout: Option<u64>,

    /// save metric file or not
    #[clap(long)]
    pub(crate) keep_metric: bool,

    #[clap(subcommand)]
    pub(crate) command: Option<Commands>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fund Ethereum accounts
    Fund {
        /// ethereum-compatible network
        #[clap(long)]
        network: String,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// block time of the network
        #[clap(long, default_value_t = BLOCK_TIME)]
        block_time: u64,

        /// the number of Eth Account to be fund
        #[clap(long, default_value_t = 0)]
        count: u64,

        /// how much 0.1-eth to fund
        #[clap(long, default_value_t = 1)]
        amount: u64,

        /// load keys from file
        #[clap(long)]
        load: bool,

        /// re-deposit account with insufficient balance
        #[clap(long)]
        redeposit: bool,
    },
    /// check ethereum account information
    Info {
        /// ethereum-compatible network
        #[clap(long)]
        network: String,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// ethereum address
        #[clap(long)]
        account: Address,
    },

    /// Transaction Operations
    Transaction {
        /// ethereum-compatible network
        #[clap(long)]
        network: String,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// transaction hash
        #[clap(long)]
        hash: H256,
    },

    /// Block Operations
    Block {
        /// ethereum-compatible network
        #[clap(long)]
        network: String,

        /// http request timeout, seconds
        #[clap(long)]
        timeout: Option<u64>,

        /// start block height
        #[clap(long)]
        start: Option<u64>,

        /// block count, could be less than zero
        #[clap(long)]
        count: Option<i64>,
    },

    /// ETL procession
    Etl {
        /// abcid log file
        #[clap(long)]
        abcid: String,

        /// tendermint log file
        #[clap(long)]
        tendermint: String,

        /// redis db address
        #[clap(long, default_value = "127.0.0.1:6379")]
        redis: String,

        /// load data
        #[clap(long)]
        load: bool,
    },
}
