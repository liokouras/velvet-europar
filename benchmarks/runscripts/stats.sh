#!/bin/bash

OUT_DIR="${1}"

THREADS=64
CORES=$(seq -s, 0 $((THREADS - 1)))
################################################################## ADAPINT ##################################################################
run_adapint() {
    local app="adapint"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/$app/out.out"
    local dump_file="$OUT_DIR/$app/dump.err"
    local s1="$OUT_DIR/$app/stats1.csv"
    local s2="$OUT_DIR/$app/stats2.csv"
    local s3="$OUT_DIR/$app/stats3.csv"
    local s4="$OUT_DIR/$app/stats4.csv"
    local s5="$OUT_DIR/$app/stats5.csv"

    echo "ADAPINT benchmark. Saving logs to $output_file"
    echo "version,num_workers,a,b,epsilon,threshold,time_secs" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"

    A=-10000
    B=4000000
    E=0.0001
    local ARGS=("$A" "$B" "$E")

    echo "running ADAPINT stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"

    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## BH ##################################################################
run_bh() {
    local app="bh"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/$app/out.out"
    local dump_file="$OUT_DIR/$app/err.err"
    local s1="$OUT_DIR/$app/stats1.csv"
    local s2="$OUT_DIR/$app/stats2.csv"
    local s3="$OUT_DIR/$app/stats3.csv"
    local s4="$OUT_DIR/$app/stats4.csv"
    local s5="$OUT_DIR/$app/stats5.csv"
    
    echo "BH benchmark. Saving logs to $output_file"
    echo "version,num_workers,bucket_size,spawn_threshold,total_time,tree_time,forces_time,bodies_time" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"

    DATA="three_plummers_4M_wider"
    INPUT="/local/badia/data/$DATA.txt"
    OUTPUT="/local/badia/data/$DATA.txt"
    ITERS=5
    local ARGS=("$INPUT" "$OUTPUT" "$ITERS")

    echo "running BH stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"
    
    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## FIB ##################################################################
run_fib() {
    local app="fib"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/$app/out.out"
    local dump_file="$OUT_DIR/$app/err.err"
    local s1="$OUT_DIR/$app/stats1.csv"
    local s2="$OUT_DIR/$app/stats2.csv"
    local s3="$OUT_DIR/$app/stats3.csv"
    local s4="$OUT_DIR/$app/stats4.csv"
    local s5="$OUT_DIR/$app/stats5.csv"
    
    echo "FIB benchmark. Saving logs to $output_file"
    echo "version,num_workers,n,threshold,time_secs" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"

    N=52
    local ARGS=("$N")

    echo "running FIB stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"

    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## MATMUL ##################################################################
run_matmul() {
    local app="matmul"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/matmul/out.out"
    local dump_file="$OUT_DIR/matmul/err.err"
    local s1="$OUT_DIR/matmul/stats1.csv"
    local s2="$OUT_DIR/matmul/stats2.csv"
    local s3="$OUT_DIR/matmul/stats3.csv"
    local s4="$OUT_DIR/matmul/stats4.csv"
    local s5="$OUT_DIR/matmul/stats5.csv"

    echo "MATMUL benchmark. Saving logs to $output_file"
    echo "version,num_workers,depth,dim,time_secs" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"

    DEPTH=7
    DIM=64
    local ARGS=("$DEPTH" "$DIM")

    echo "running MATMUL stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"

    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## NQUEENS ##################################################################
run_nqueens() {
    local app="nqueens"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/$app/out.out"
    local dump_file="$OUT_DIR/$app/err.err"
    local s1="$OUT_DIR/$app/stats1.csv"
    local s2="$OUT_DIR/$app/stats2.csv"
    local s3="$OUT_DIR/$app/stats3.csv"
    local s4="$OUT_DIR/$app/stats4.csv"
    local s5="$OUT_DIR/$app/stats5.csv"

    echo "NQUEENS benchmark. Saving logs to $output_file"
    echo "version,num_workers,n,threshold,time_secs" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"


    N=16
    local ARGS=("$N")

    echo "running NQUEENS stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"

    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## SORT ##################################################################
run_sort() {
    local app="sort"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/$app/out.out"
    local dump_file="$OUT_DIR/$app/err.err"
    local s1="$OUT_DIR/$app/stats1.csv"
    local s2="$OUT_DIR/$app/stats2.csv"
    local s3="$OUT_DIR/$app/stats3.csv"
    local s4="$OUT_DIR/$app/stats4.csv"
    local s5="$OUT_DIR/$app/stats5.csv"


    echo "SORT benchmark. Saving logs to $output_file"
    echo "version,num_workers,threshold,array_length,random_seed,time_secs" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"


    N=2000000000
    SEED=42
    local ARGS=("$N" "$SEED")

    echo "running SORT stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"

    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## TSP ##################################################################
run_tsp() {
    local app="tsp"
    mkdir -p "$OUT_DIR/$app"
    cd ../rust/$app/
    local output_file="$OUT_DIR/$app/out.out"
    local dump_file="$OUT_DIR/$app/err.err"
    local s1="$OUT_DIR/$app/stats1.csv"
    local s2="$OUT_DIR/$app/stats2.csv"
    local s3="$OUT_DIR/$app/stats3.csv"
    local s4="$OUT_DIR/$app/stats4.csv"
    local s5="$OUT_DIR/$app/stats5.csv"

    echo "TSP benchmark. Saving logs to $output_file"
    echo "version,num_workers,ntowns,seed,seq_threshold,time_secs" > "$output_file"

    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s1"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s2"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s3"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s4"
    echo "thread_id,total_steal_attempts,successful_steals,attempts_before_first_success,work_time,steal_setup_time,steal_waiting,pop_waiting,pop_waiting_other,push_waiting,push_waiting_other,other_time,spawns,spawns_other,total_stolen_jobs,total_stolen_jobs_other,sync_loop_iters,sync_loop_iters_other" > "$s5"

    N=19
    SEED=25
    local ARGS=("$N" "$SEED")

    echo "running TSP stat collection with Velvet using $THREADS threads"
    export VELVET_WORKERS=$THREADS 
    # warmup
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$dump_file"
    # real
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s1"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s2"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s3"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s4"
    taskset -c "$CORES"  cargo run --release --features "stats" velvet "${ARGS[@]}" >> "$output_file" 2>> "$s5"

    awk 'NR==1 || /^[0-9]/' "$s1" > tmp.csv && mv tmp.csv "$s1"
    awk 'NR==1 || /^[0-9]/' "$s2" > tmp.csv && mv tmp.csv "$s2"
    awk 'NR==1 || /^[0-9]/' "$s3" > tmp.csv && mv tmp.csv "$s3"
    awk 'NR==1 || /^[0-9]/' "$s4" > tmp.csv && mv tmp.csv "$s4"
    awk 'NR==1 || /^[0-9]/' "$s5" > tmp.csv && mv tmp.csv "$s5"

    cd - > /dev/null
}

################################################################## EXPERIMENTING ##################################################################

run_adapint
run_bh
run_fib
run_matmul
run_nqueens
run_sort
run_tsp