#!/bin/bash

THREADS_FULL=(1 2 4 8 16 32 48 64 80 96 112 128)
THREADS_CHECK=(16 32)
ACTIVE_THREADS=("${THREADS_CHECK[@]}")

ITERS_FULL=(1 1 1 1 1 1)
ITERS_CHECK=(1 1 1)
ACTIVE_ITERS=("${ITERS_CHECK[@]}")

HAVE_RUST="$3"
HAVE_CLANG_OMP="$4"
HAVE_OPENCILK="$5"

APP="sort"
OUT="${1}/${APP}.csv"
DUMP="${2}/sort.dump"

echo "SORT benchmark. Saving logs to $OUT"
echo "version,num_workers,threshold,array_length,random_seed,time_secs" > "$OUT"

# TODO ADJUST!
N=20000000
SEED=42

if [ "$HAVE_RUST" = "true" ]; then
    cd ../rust/sort/

    echo "running SORT serial Rust"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release seq $N $SEED >> "$OUT" 2>> "$DUMP"
    done

    echo "running SORT serial elision"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release par_seq $N $SEED >> "$OUT" 2>> "$DUMP"
    done

    echo "running SORT rayon"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release --features "rayon" rayon $N $SEED $threads >> "$OUT" 2>> "$DUMP"
        done
    done

    echo "running SORT velvet"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        export VELVET_WORKERS=$threads 
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release velvet $N $SEED >> "$OUT" 2>> "$DUMP"
        done
    done

    echo "running SORT Velvet with test_direct"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_direct_rec" test_direct $N $SEED 1 >> "$OUT" 2>> "$DUMP"
    done

    cd ../sort_unsafe/
    echo "running SORT-UNSAFE serial elision"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release par_seq $N $SEED >> "$OUT" 2>> "$DUMP"
    done

    echo "running SORT-UNSAFE velvet"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        export VELVET_WORKERS=$threads 
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release velvet $N $SEED >> "$OUT" 2>> "$DUMP"
        done
    done

    # create data for C-applications to use
    cargo run --release gen_arr $N $SEED >> "$DUMP" 2>> "$DUMP"

    cd - > /dev/null
fi

if [ "$HAVE_CLANG_OMP" = "true" ]; then
    cd ../c/

    # ensure Rust sort is compiled
    cd sort_rs
    cargo build --release
    cd ..

    make -C ./openmp/sort/

    echo "running SORT serial C"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 "./zout/${APP}_omp" seq $N $SEED >> "$OUT" 2>> "$DUMP"
    done

    echo "running SORT openmp"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        export OMP_NUM_THREADS=$threads
        export OMP_PROC_BIND=true
        export OMP_PLACES=cores
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" "./zout/${APP}_omp" omp $N $SEED >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi

if [ "$HAVE_OPENCILK" = "true" ]; then
    #TODO
    echo "cilk coming!"
fi