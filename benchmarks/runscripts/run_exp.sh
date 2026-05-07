#!/bin/bash

# ** MUST RUN FROM INSIDE RUNSCRIPTS FOLDER **

# create timestamped dirs
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "../output/out_$TIMESTAMP"
mkdir -p "../output/dump_$TIMESTAMP"
OUT_DIR="$(realpath "../output/out_$TIMESTAMP")"
DUMP_DIR="$(realpath "../output/dump_$TIMESTAMP")"

# ensure existence of zout and data folders
mkdir -p "../c/zout"
mkdir -p "../data"

# check for dependencies
HAVE_RUST=false
HAVE_CLANG_OMP=false
HAVE_OPENCILK=false

command -v rustc &>/dev/null && command -v cargo &>/dev/null && HAVE_RUST=true
command -v clang &>/dev/null && echo | clang -fopenmp -x c -c -o /dev/null - &>/dev/null 2>&1 && HAVE_CLANG_OMP=true
# command -v clang &>/dev/null && clang --version 2>&1 | grep -qi "opencilk" && HAVE_OPENCILK=true

if [ "$HAVE_RUST" = "false" ]; then
    echo "WARNING: Rust and Cargo not found. These are necessary to run the fundamental benchmarks."
fi
if [ "$HAVE_CLANG_OMP" = "false" ]; then
    echo "WARNING: clang with OpenMP not found. This is necessary for the comparison with OpenMP. Will be skipped..."
fi
if [ "$HAVE_OPENCILK" = "false" ]; then
    echo "WARNING: OpenCilk not found. This is necessary for the comparison with Cilk. Will be skipped..."
fi

# pass output dir and envr-info to all (or a subset?) of the app-specific scripts
# take also as arg the max nr of threads to test? to then either set it as an envr var or pass to the app-specific scripts?

bash adapint.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST" "$HAVE_CLANG_OMP" "$HAVE_OPENCILK"

bash bh.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST" "$HAVE_CLANG_OMP" "$HAVE_OPENCILK"

bash fib.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST"

bash matmul.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST" "$HAVE_CLANG_OMP" "$HAVE_OPENCILK"

bash nqueens.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST" "$HAVE_CLANG_OMP" "$HAVE_OPENCILK"

bash sort.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST" "$HAVE_CLANG_OMP" "$HAVE_OPENCILK"

bash tsp.sh "$OUT_DIR" "$DUMP_DIR" "$HAVE_RUST" "$HAVE_CLANG_OMP" "$HAVE_OPENCILK"


# STATS !!!

# pass data dir to processing script
