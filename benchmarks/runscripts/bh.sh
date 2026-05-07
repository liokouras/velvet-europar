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

APP="bh"
OUT="${1}/${APP}.csv"
DUMP="${2}/bh.dump"

echo "BH benchmark. Saving logs to $OUT"
echo "version,num_workers,bucket_size,spawn_threshold,total_time,tree_time,forces_time,bodies_time" > "$OUT"

#TODO adjust...
DATA="three_plummers_4M_wider"
INPUT="/local/badia/data/$DATA.txt"
OUTPUT="/local/badia/data/$DATA.txt"
ITERS=5

if [ "$HAVE_RUST" = "true" ]; then
    cd ../rust/bh/

    echo "running BH serial Rust"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release seq $INPUT $OUTPUT $ITERS >> "$OUT" 2>> "$DUMP"
    done

    echo "running BH serial elision"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release par_seq $INPUT $OUTPUT $ITERS >> "$OUT" 2>> "$DUMP"
    done

    for version in "rayon_pariter" "rayon_iterative" "rayon_treeiter"
    do
        echo "running BH $version"
        for threads in "${ACTIVE_THREADS[@]}"
        do
            CORES=$(seq -s, 0 $((threads - 1)))
            for iter in "${ACTIVE_ITERS[@]}"; do
                taskset -c "$CORES" cargo run --release --features "rayon" $version $INPUT $OUTPUT $ITERS $threads >> "$OUT" 2>> "$DUMP"
            done
        done
    done

    echo "running BH velvet"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        export VELVET_WORKERS=$threads 
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release velvet $INPUT $OUTPUT $ITERS >> "$OUT" 2>> "$DUMP"
        done
    done

    echo "running BH Velvet with test_direct"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_direct_rec" test_direct $INPUT $OUTPUT $ITERS 1 >> "$OUT" 2>> "$DUMP"
    done
    
    cd - > /dev/null
fi


if [ "$HAVE_CLANG_OMP" = "true" ]; then
    cd ../c/

    clang -fopenmp -O3 "./openmp/$APP/${APP}.c" "./openmp/$APP/quad_body.c" "./openmp/$APP/driver.c" -lm -o "./zout/${APP}_omp"

    echo "running BH serial C"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 "./zout/${APP}_omp" seq $INPUT $OUTPUT $ITERS >> "$OUT" 2>> "$DUMP"
    done


    echo "running BH openmp"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        export OMP_NUM_THREADS=$threads
        export OMP_PROC_BIND=true
        export OMP_PLACES=cores
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" "./zout/${APP}_omp" omp $INPUT $OUTPUT $ITERS >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi

if [ "$HAVE_OPENCILK" = "true" ]; then
    cd ../c/

    $OPENCILK_HOME/bin/clang -L$OPENCILK_HOME/lib -L$OPENCILK_HOME/lib64 -fopencilk -O3 "./cilk/$APP/${APP}.c" "./cilk/$APP/quad_body.c" "./cilk/$APP/driver.c" -lm -o "./zout/${APP}_cilk"

    echo "running BH cilk"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        export CILK_NWORKERS=$threads 
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" "./zout/${APP}_cilk" cilk $INPUT $OUTPUT $ITERS >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi