#!/usr/bin/env bash

set -e

test_endpoint="http://dev-qa01-us-west-2-full-001-open.dev.findora.org:8545"

#cd "$script_dir"/..

run_one_test() {
    endpoint=$1
    count=$2
    concurrency=$3
    timeout=$4
    wait=$6
    logfile=$5

    {
        echo "$endpoint $count $concurrency $timeout"
        RUST_LOG=info ./feth --network "$endpoint" --count "$count" --max-parallelism "$concurrency" --timeout "$timeout" 2>&1
        echo "wait for 60 seconds to make sure all previous txns have been finished..."
    } >>"$logfile"
    sleep "$wait"
}

for ((i = 0; i < 1000; i++)); do
    run_one_test "$test_endpoint" 1 100 100 1 test.log
done
