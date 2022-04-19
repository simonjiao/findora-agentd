#!/usr/bin/env bash

set -e

endpoint00="http://dev-qa01-us-west-2-full-000-open.dev.findora.org:8545"
endpoint01="http://dev-qa01-us-west-2-full-001-open.dev.findora.org:8545"

#cd "$script_dir"/..

run_one_test() {
    endpoint=$1
    count=$2
    concurrency=$3
    timeout=$4
    waittime=$5
    source=$6
    logfile=$7

    {
        echo "$endpoint $count $concurrency $timeout"
        RUST_LOG=info ./feth --network "$endpoint" --source "$source" --count "$count" --max-parallelism "$concurrency" --timeout "$timeout" 2>&1
        echo "wait for 60 seconds to make sure all previous txns have been finished..."
    } >>"$logfile"
    sleep "$waittime"
}

for ((i = 0; i < 10000; i++)); do
    run_one_test "$endpoint00" 1 300 100 1 "source_keys.001" "test00.log"
done &

for ((i = 0; i < 10000; i++)); do
    run_one_test "$endpoint01" 1 300 100 1 "source_keys.002" "test01.log"
done
