#!/usr/bin/env bash

set -e

# build binary for testing
cargo build --release

# Save the faucet private key to `.secret`
echo "4c10030f9fc32db7bbf15e8b527823a4083ab36d7b525b98e6a3f01d875960cc" >.secret

# Generate some accounts and transfer some Ethers (for example 1000*0.1 ether) to them
cargo run --release -- fund --network qa01 --count 100 --amount 1000

# The accounts used for testing will be saved to "sources_keys.001"
less sources_keys.001

# re-deposit the account whose balance is lower than the specified amount
cargo run --release -- fund --network qa01 --amount 1000 --load --redeposit --timeout 5

# Add more accounts and re-deposit them
cargo run --release -- fund --network qa01 --count 200 --amount 1000 --load --redeposit

# Starting tests
cargo run --release -- --network qa01 --count 10 --max-parallism 200 --timeout 10
# 1. Load source accounts from "source_keys.001"
# 2. Filter out account which doesn't have sufficient balance
# 3. Generate the "count" of new addresses per source account to receive Ethers
# 4. Create a thread pool with size of "max-parallelism", thread pool size could be larger than source keys' count
# 5. Build transactions and send them to the endpoints
#   a. One context for each source account
#   b. For one source account, we will build tx, sign it, then send it, and wait for the receipt, 3*block_time maximum.
#   c. All test results will save to files with prefix "metric.target.*"

# Collect test results for further analysis
mkdir test_results
mv metric.target.* test_results
mv metric.001 test_results

# Backup your source_keys file
cp source_keys.001 ~/source_keys.xx.200

# Specify source_keys file
cargo run --release -- --network qa01 --count 10 --max-parallelism 200 --source ~/source_keys.xx.200

# Multi endpoints seperated by comma
# The source account will be divided equally to each endpoint
cargo run --release -- --network http://localhost:8545,http://localhost:8555 --count 10 --max-parallelism 200

# Retrieve transaction by hash
cargo run --release -- transaction --network qa01 --hash 0x1d44bd3fc1764c6dfadb6eef7191cf44a81607c02c41255f7802f4779de55dcf

# Retrieve account basic information, such as `balance` and `nonce`
cargo run --release -- info --network qa01 --account 0x512a4d5e8478D11682925b29705F6c8d6AE9e39d

# Retrieve block information, such as height, timestamp, txn counts, time interval
# show current block information
cargo run --release -- block --network qa01
# 11297,1646890818,0,16
# Show evm blocks since specified height
cargo run --release -- block --network qa01 --start 11290
# 11290,1646890708,0,16
# 11291,1646890724,0,16
# 11292,1646890740,0,16
# 11293,1646890755,0,15
# 11294,1646890771,0,16
# 11295,1646890787,0,16
# 11296,1646890802,0,15
# 11297,1646890818,0,16
# 11298,1646890834,0,16
# Show tendermint blocks of qa01 since specified height, used this command if you cares about all not just evm txns
bash fetch_block.sh 11290

# test contract deploy
cargo run --release -- contract --optype deploy --network test --config ~/contract_deploy.json

# test contract call
cargo run --release -- contract --optype call --network test --config ~/contract_call.json

# test contract query
cargo run --release -- contract --optype query --network test --config ~/contract_query.json