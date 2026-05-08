#!/bin/bash

HAVE_RUST="$3"
RUN_MODE="$7"
APP="fib"
OUT="${1}/${APP}.csv"
DUMP="${2}/fib.dump"

ITERS_FULL=(1 1 1 1 1 1)
ITERS_REDUCED=(1 1 1)

if [ "$RUN_MODE" = "full" ]; then
    ACTIVE_ITERS=("${ITERS_FULL[@]}")
    N=52
else
    ACTIVE_ITERS=("${ITERS_REDUCED[@]}")
    N=48
fi

echo "FIB benchmark. Saving logs to $OUT"
echo "version,num_workers,n,threshold,time_secs" > "$OUT"

if [ "$HAVE_RUST" = "true" ]; then
    cd ../rust/fib/

    echo "running FIB serial Rust"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release seq $N >> "$OUT" 2>> "$DUMP"
    done

    echo "running FIB velvet"
    export VELVET_WORKERS=1 
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release velvet $N >> "$OUT" 2>> "$DUMP"
    done

    echo "running FIB Velvet with test_direct"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_direct_rec" test_direct $N 1 >> "$OUT" 2>> "$DUMP"
    done

    echo "running FIB Velvet with test_direct without a serial threshold"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_direct_rec, test_no_thresh" test_direct $N 1 >> "$OUT" 2>> "$DUMP"
    done

    echo "running FIB Velvet without a serial threshold"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_no_thresh" velvet $N 1 >> "$OUT" 2>> "$DUMP"
    done
    
    cd - > /dev/null
fi