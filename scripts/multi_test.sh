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
        echo "wait for $waittime seconds to make sure all previous txns have been finished..."
    } >>"$logfile"
    sleep "$waittime"
}

switch_profiler() {
    enable=$1
    logfile=$2

    if [ "$enable" ]; then
        RUST_LOG=info ./feth profiler --network "$endpoint" --enable 2>&1 >>"$logfile"
    else
        RUST_LOG=info ./feth profiler --network "$endpoint" 2>&1 >>"$logfile"
    fi

}

switch_profiler true "test01.log"

for ((i = 0; i < 10000; i++)); do
    run_one_test "$endpoint01" 1 200 100 5 "source_keys.001" "test01.log"
done

switch_profiler false "test01.log"

#for ((i = 0; i < 10000; i++)); do
#    run_one_test "$endpoint01" 1 300 100 1 "source_keys.002" "test02.log"
#done &
#
#for ((i = 0; i < 10000; i++)); do
#    run_one_test "$endpoint00" 1 300 100 1 "source_keys.003" "test03.log"
#done &
#
#for ((i = 0; i < 10000; i++)); do
#    run_one_test "$endpoint01" 1 300 100 1 "source_keys.004" "test04.log"
#done
