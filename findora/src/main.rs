use clap::{Parser, Subcommand};
use std::ops::Mul;
use std::str::FromStr;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use feth::{one_eth_key, KeyPair, TestClient, TransferMetrics, BLOCK_TIME, ROOT_ADDR};
use web3::types::Address;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    /// The minimum parallelism
    #[clap(long, default_value_t = 10)]
    min_parallelism: u64,

    /// The maximum parallelism
    #[clap(long, default_value_t = 2000)]
    max_parallelism: u64,

    /// The count of transactions sent by a routine
    #[clap(long, default_value_t = 20)]
    count: u64,

    /// load source accounts from file, or generate new accounts
    #[clap(long)]
    load: bool,

    /// the source account file
    #[clap(long, parse(from_os_str), value_name = "FILE", default_value = "source_keys.001")]
    source: PathBuf,

    /// block time of the network
    #[clap(long, default_value_t = BLOCK_TIME)]
    block_time: u64,

    /// findora network fullnode urls: http://path:8545,http://path1:8546
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fund Ethereum accounts
    Fund {
        /// the number of Eth Account to te fund
        #[clap(long)]
        count: u64,
        /// fund amount
        #[clap(long)]
        amount: Option<u64>,
    },
}

fn main() -> web3::Result<()> {
    let cli = Cli::parse();

    println!("{:?}", cli);

    let per_count = cli.count;
    let min_par = cli.min_parallelism;
    let _max_par = cli.max_parallelism;
    let source_file = cli.source;
    let _prog = "feth".to_owned();
    let mut source_keys = None;
    let mut metrics = None;
    let block_time = Some(cli.block_time);
    if cli.load {
        let keys: Vec<KeyPair> = serde_json::from_str(std::fs::read_to_string(source_file).unwrap().as_str()).unwrap();
        source_keys = Some(keys);
    } else {
        println!("generating new source keys");
        if std::fs::File::open("source_keys.001").is_ok() {
            panic!("file \"source_keys.001\" already exists");
        }
    }
    let source_amount = web3::types::U256::exp10(18 + 3); // 1000 eth
    let target_amount = web3::types::U256::exp10(17); // 0.1 eth

    let client = TestClient::setup(cli.network, None, None);

    println!("chain_id:     {}", client.chain_id().unwrap());
    println!("gas_price:    {}", client.gas_price().unwrap());
    println!("block_number: {}", client.block_number().unwrap());
    println!("frc20 code:   {:?}", client.frc20_code().unwrap());
    let balance = client.balance(ROOT_ADDR[2..].parse().unwrap(), None);
    println!("Root Balance: {}", balance);

    let source_keys = source_keys.unwrap_or_else(|| {
        // generate new keys
        if std::fs::File::open("source_keys.001").is_ok() {
            panic!("file \"source_keys.001\" already exists");
        }
        if source_amount.mul(min_par + 1) >= balance {
            panic!("Too large source account number, maximum {}", balance / source_amount);
        }
        let source_keys = (0..min_par).map(|_| one_eth_key()).collect::<Vec<_>>();
        let data = serde_json::to_string(&source_keys).unwrap();
        std::fs::write("source_keys.001", &data).unwrap();

        let source_accounts = source_keys.iter().map(|key| key.address.as_str()).collect::<Vec<_>>();
        // 1000 eth
        let amounts = vec![source_amount; min_par as usize];
        metrics = Some(
            client
                .distribution(None, &source_accounts, &amounts, block_time)
                .unwrap()
                .0,
        );
        // save metrics to file
        let data = serde_json::to_string(&metrics).unwrap();
        std::fs::write("metrics.001", &data).unwrap();

        source_keys
    });
    let metrics = metrics.unwrap_or_else(|| {
        source_keys
            .iter()
            .filter_map(|kp| {
                let balance = client.balance(kp.address[2..].parse().unwrap(), None);
                if balance <= target_amount.mul(per_count) {
                    None
                } else {
                    Some(TransferMetrics {
                        from: client.root_addr,
                        to: Default::default(),
                        amount: balance,
                        hash: None,
                        status: 1,
                        wait: 0,
                    })
                }
            })
            .collect::<Vec<_>>()
    });

    if min_par == 0 || per_count == 0 || metrics.is_empty() {
        return Ok(());
    }

    let client = Arc::new(client);
    let mut handles = vec![];
    let total_succeed = Arc::new(Mutex::new(0u64));
    let now = std::time::Instant::now();

    metrics.into_iter().enumerate().for_each(|(i, m)| {
        if m.status == 1 {
            let client = client.clone();
            let target_count = per_count;
            let keys = (0..target_count).map(|_| one_eth_key()).collect::<Vec<_>>();
            let am = target_amount;
            let source = source_keys.get(i).map(|s| {
                (
                    secp256k1::SecretKey::from_str(s.private.as_str()).unwrap(),
                    Address::from_str(s.address.as_str()).unwrap(),
                )
            });
            let total_succeed = total_succeed.clone();

            let handle = thread::spawn(move || {
                let amounts = vec![am; target_count as usize];
                let accounts = keys.iter().map(|key| key.address.as_str()).collect::<Vec<_>>();
                let (metrics, succeed) = client.distribution(source, &accounts, &amounts, block_time).unwrap();
                let file = format!("metrics.target.{}", i);
                let data = serde_json::to_string(&metrics).unwrap();
                std::fs::write(file, data).unwrap();

                let mut num = total_succeed.lock().unwrap();
                *num += succeed;
            });
            handles.push(handle);
        }
    });

    let source_count = handles.len();
    for h in handles {
        h.join().unwrap();
    }

    let elapsed = now.elapsed().as_secs();
    let avg = source_count as f64 * per_count as f64 / elapsed as f64;
    println!(
        "Transfer from {} accounts to {} accounts concurrently, succeed {}, {:.3} Transfer/s, total {} seconds",
        source_count,
        per_count,
        total_succeed.lock().unwrap(),
        avg,
        elapsed,
    );

    Ok(())
}
