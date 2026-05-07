#!/bin/bash

THREADS_FULL=(1 2 4 8 16 32 48 64 80 96 112 128)
ACTIVE_THREADS=()
for t in "${THREADS_FULL[@]}"; do
    if [ "$t" -le "$6" ]; then
        ACTIVE_THREADS+=("$t")
    fi
done

ITERS_FULL=(1 1 1 1 1 1)
ITERS_CHECK=(1 1 1)
ACTIVE_ITERS=("${ITERS_CHECK[@]}")

HAVE_RUST="$3"
HAVE_CLANG_OMP="$4"
HAVE_OPENCILK="$5"

APP="tsp"
OUT="${1}/${APP}.csv"
DUMP="${2}/tsp.dump"

echo "TSP benchmark. Saving logs to $OUT"
echo "version,num_workers,ntowns,seed,seq_threshold,time_secs" > "$OUT"

# TODO ADJUST!
N=19
SEED=25

if [ "$HAVE_RUST" = "true" ]; then

    cd ../rust/tsp/

    echo "running TSP serial Rust"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release seq $N $SEED >> "$OUT" 2>> "$DUMP"
    done

    echo "running TSP rayon"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release --features "rayon" rayon $N $SEED $threads >> "$OUT" 2>> "$DUMP"
        done
    done

    echo "running TSP velvet"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        export VELVET_WORKERS=$threads 
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release velvet $N $SEED >> "$OUT" 2>> "$DUMP"
        done
    done

    echo "running TSP Velvet with test_direct"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_direct_rec" test_direct $N $SEED 1 >> "$OUT" 2>> "$DUMP"
    done

    # create data for C-applications to use
    cargo run --release gen_towns $N $SEED >> "$DUMP" 2>> "$DUMP"

    cd - > /dev/null
fi

if [ "$HAVE_CLANG_OMP" = "true" ]; then
    cd ../c/

    clang -O3 "./openmp/$APP/tsp_seq.c" "./openmp/$APP/distance_table.c" -lm -o "./zout/${APP}_seq"

    echo "running TSP serial C"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 "./zout/${APP}_seq" seq $N $SEED >> "$OUT" 2>> "$DUMP"
    done

    clang -fopenmp -O3 "./openmp/$APP/${APP}.c" "./openmp/$APP/distance_table.c" -lm -o "./zout/${APP}_omp"

    echo "running TSP openmp"
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
    cd ../c/

    $OPENCILK_HOME/bin/clang -L$OPENCILK_HOME/lib -L$OPENCILK_HOME/lib64 -fopencilk -O3 "./cilk/$APP/${APP}.c" "./cilk/$APP/distance_table.c" -lm -o "./zout/${APP}_cilk"

    echo "running TSP cilk"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        export CILK_NWORKERS=$threads 
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" "./zout/${APP}_cilk" cilk $N $SEED >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi