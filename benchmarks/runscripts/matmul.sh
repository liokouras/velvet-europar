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

APP="matmul"
OUT="${1}/${APP}.csv"
DUMP="${2}/matmul.dump"

echo "MATMUL benchmark. Saving logs to $OUT"
echo "version,num_workers,depth,dim,time_secs" > "$OUT"

#TODO adjust!
DEPTH=7
DIM=64

if [ "$HAVE_RUST" = "true" ]; then
    cd ../rust/matmul/

    echo "running MATMUL serial Rust"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release seq $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
    done

    echo "running MATMUL serial elision"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release par $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
    done

    for version in "rayon" "rayon-z" "rayon-strassen"
    do
        echo "running MATMUL $version"
        for threads in "${ACTIVE_THREADS[@]}"
        do
            CORES=$(seq -s, 0 $((threads - 1)))
            for iter in "${ACTIVE_ITERS[@]}"; do
                taskset -c "$CORES" cargo run --release --features "rayon" $version $DEPTH $DIM $threads >> "$OUT" 2>> "$DUMP"
            done
        done
    done

    echo "running MATMUL velvet"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        export VELVET_WORKERS=$threads 
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release velvet $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
        done
    done

    echo "running MATMUL Velvet with test_direct"
    export VELVET_WORKERS=1
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release --features "test_direct_rec" test_direct $DEPTH $DIM 1 >> "$OUT" 2>> "$DUMP"
    done

    cd - > /dev/null
    cd ../rust/matmul_unsafe/

    echo "running MATMUL-UNSAFE serial Rust"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release seq $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
    done

    echo "running MATMUL-UNSAFE serial elision"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 cargo run --release par $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
    done

    echo "running MATMUL-UNSAFE velvet"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        CORES=$(seq -s, 0 $((threads - 1)))
        export VELVET_WORKERS=$threads 
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" cargo run --release velvet $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi


if [ "$HAVE_CLANG_OMP" = "true" ]; then
    cd ../c/

    clang -fopenmp -O3 "./openmp/$APP/${APP}.c" -lm -o "./zout/${APP}_omp"

    echo "running MATMUL serial C"
    for iter in "${ACTIVE_ITERS[@]}"; do
        taskset -c 0 "./zout/${APP}_omp" seq $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
    done


    echo "running MATMUL openmp"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        export OMP_NUM_THREADS=$threads
        export OMP_PROC_BIND=true
        export OMP_PLACES=cores
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" "./zout/${APP}_omp" omp $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi

if [ "$HAVE_OPENCILK" = "true" ]; then
    cd ../c/

    $OPENCILK_HOME/bin/clang -L$OPENCILK_HOME/lib -L$OPENCILK_HOME/lib64 -fopencilk -O3 "./cilk/$APP/${APP}.c" -lm -o "./zout/${APP}_cilk"

    echo "running MATMUL cilk"
    for threads in "${ACTIVE_THREADS[@]}"
    do
        export CILK_NWORKERS=$threads 
        CORES=$(seq -s, 0 $((threads - 1)))
        for iter in "${ACTIVE_ITERS[@]}"; do
            taskset -c "$CORES" "./zout/${APP}_cilk" cilk $DEPTH $DIM >> "$OUT" 2>> "$DUMP"
        done
    done

    cd - > /dev/null
fi
