use bip0039::{Count, Language, Mnemonic};
use bip32::{DerivationPath, XPrv};
use libsecp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use std::str::FromStr;
use tokio::runtime::Runtime;
use web3::transports::Http;
use web3::types::{Address, BlockNumber, Bytes, TransactionId, TransactionParameters, H160, H256, U256, U64};

const FRC20_ADDRESS: u64 = 0x1000;

//const WEB3_SRV: &str = "http://127.0.0.1:8545";
const WEB3_SRV: &str = "http://18.236.205.22:8545";
//const WEB3_SRV: &str = "https://prod-testnet.prod.findora.org:8545";
//const WEB3_SRV: &str = "https://dev-mainnetmock.dev.findora.org:8545";

const ROOT_SK: &str = "b8836c243a1ff93a63b12384176f102345123050c9f3d3febbb82e3acd6dd1cb";
const ROOT_ADDR: &str = "0xBb4a0755b740a55Bf18Ac4404628A1a6ae8B6F8F";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KeyPair {
    address: String,
    private: String,
}

fn one_eth_key() -> KeyPair {
    let mnemonic = Mnemonic::generate_in(Language::English, Count::Words12);
    let bs = mnemonic.to_seed("");
    let ext = XPrv::derive_from_path(&bs, &DerivationPath::from_str("m/44'/60'/0'/0/0").unwrap()).unwrap();

    let secret = SecretKey::parse_slice(&ext.to_bytes()).unwrap();
    let public = PublicKey::from_secret_key(&secret);

    let mut res = [0u8; 64];
    res.copy_from_slice(&public.serialize()[1..65]);
    let public = H160::from(H256::from_slice(Keccak256::digest(&res).as_slice()));

    KeyPair {
        address: eth_checksum::checksum(&format!("{:?}", public)),
        private: hex::encode(secret.serialize()),
    }
}

struct TestCLient {
    web3: web3::Web3<Http>,
    root_sk: secp256k1::SecretKey,
    rt: Runtime,
}

impl TestCLient {
    pub fn setup(url: Option<&str>, root_sk: Option<&str>) -> Self {
        let transport = web3::transports::Http::new(url.unwrap_or(WEB3_SRV)).unwrap();
        let web3 = web3::Web3::new(transport);
        let root_sk = secp256k1::SecretKey::from_str(root_sk.unwrap_or(ROOT_SK)).unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        Self { web3, root_sk, rt }
    }

    pub fn chain_id(&self) -> Option<U256> {
        self.rt.block_on(self.web3.eth().chain_id()).ok()
    }

    pub fn block_number(&self) -> Option<U64> {
        self.rt.block_on(self.web3.eth().block_number()).ok()
    }

    pub fn gas_price(&self) -> Option<U256> {
        self.rt.block_on(self.web3.eth().gas_price()).ok()
    }

    pub fn frc20_code(&self) -> Option<Bytes> {
        self.rt
            .block_on(self.web3.eth().code(H160::from_low_u64_be(FRC20_ADDRESS), None))
            .ok()
    }

    #[allow(unused)]
    pub fn accounts(&self) -> Vec<Address> {
        self.rt.block_on(self.web3.eth().accounts()).unwrap_or_default()
    }

    pub fn balance(&self, address: Address, number: Option<BlockNumber>) -> U256 {
        self.rt
            .block_on(self.web3.eth().balance(address, number))
            .unwrap_or_default()
    }

    pub fn distribution(&self, accounts: &[&str], amounts: &[u64]) -> web3::Result<()> {
        let results = accounts
            .iter()
            .zip(amounts)
            .map(|(&account, &am)| TransactionParameters {
                to: Some(Address::from_str(account).unwrap()),
                value: U256::from(am),
                ..Default::default()
            })
            // Sign the txs (can be done offline)
            .filter_map(|tx_object| {
                self.rt
                    .block_on(self.web3.accounts().sign_transaction(tx_object, &self.root_sk))
                    .ok()
            })
            // Send the txs to infra
            .filter_map(|signed| {
                self.rt
                    .block_on(self.web3.eth().send_raw_transaction(signed.raw_transaction))
                    .ok()
            })
            .collect::<Vec<_>>();

        println!("Tx succeeded with hash: {}", results.len());
        results.into_iter().for_each(|result| {
            println!(
                "Tx receipt: {:?}",
                self.rt.block_on(self.web3.eth().transaction_receipt(result))
            );
            println!(
                "Tx {:?}",
                self.rt
                    .block_on(self.web3.eth().transaction(TransactionId::Hash(result)))
            )
        });

        Ok(())
    }
}

fn main() -> web3::Result<()> {
    let client = TestCLient::setup(None, None);

    println!("chain_id {}", client.chain_id().unwrap());
    println!("gas_price {}", client.gas_price().unwrap());
    println!("block_number {}", client.block_number().unwrap());
    println!("frc20 code {:?}", client.frc20_code().unwrap());
    println!("Calling balance.");
    let balance = client.balance(ROOT_ADDR[2..].parse().unwrap(), None);
    println!("Balance of ROOT: {}", balance);

    let keys = (0..20).map(|_| one_eth_key()).collect::<Vec<_>>();
    let accounts = keys.iter().map(|key| key.address.as_str()).collect::<Vec<_>>();
    let amounts = vec![U256::exp10(17).as_u64(); 20];
    client.distribution(&accounts, &amounts)?;

    for account in accounts {
        let balance = client.balance(account.parse().unwrap(), None);
        println!("Balance of {:?}: {}", account, balance);
    }

    Ok(())
}
