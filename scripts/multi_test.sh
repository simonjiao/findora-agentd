#!/usr/bin/env bash

set -e

endpoint="https://dev-qa01.dev.findora.org"
port=26657

script_dir=$(dirname "$0")
. "$script_dir"/common.sh

test_endpoint="http://dev-qa01-us-west-2-sentry-001-open.dev.findora.org:8545"

cd "$script_dir"/..

#./feth --network "$test_endpoint" --count 2000 --max-parallelism 10 --timeout 100 >> test.log 2>&1
#echo "wait for 60 seconds to make sure all previous txns have been finished..." >> test.log
#sleep 60

./feth --network "$test_endpoint" --count 2000 --max-parallelism 15 --timeout 100 >> test.log 2>&1
echo "wait for 60 seconds to make sure all previous txns have been finished..." >> test.log
sleep 60

./feth --network "$test_endpoint" --count 2000 --max-parallelism 20 --timeout 100 >> test.log 2>&1
echo "wait for 60 seconds to make sure all previous txns have been finished..." >> test.log
sleep 60

./feth --network "$test_endpoint" --count 2000 --max-parallelism 25 --timeout 100 >> test.log 2>&1
echo "wait for 60 seconds to make sure all previous txns have been finished..." >> test.log
sleep 60

./feth --network "$test_endpoint" --count 2000 --max-parallelism 30 --timeout 100 >> test.log 2>&1
echo "wait for 60 seconds to make sure all previous txns have been finished..." >> test.log
sleep 60

cd -
