COMPILE_APPS = ["adapint", "bh", "fib", "matmul", "sort", "sort-unsafe", "tsp", "nqueens"]
APP_NAMES=["fib", "adapint", "tsp", "nqueens", "bh", "matmul", "sort"]

LABEL_VERSION= [(2,"Velvet"), (1,"Rayon")]
LABELS_MATMUL = [(2,"Velvet"), (1,"Rayon, Ours"), (3, "Velvet, Unsafe App."), (5,"Rayon, Demo-Z"), (6,"Rayon, Demo-Strassen")]
LABELS_BH = [(2,"Velvet",), (5,"Rayon, ParIter"), (1,"Rayon, TreeIter")]
LABELS_SORT = [(2,"Velvet"), (1,"Rayon"), (3, "Velvet, Unsafe App.")]
LABELS = {
    "adapint": LABEL_VERSION,
    "tsp": LABEL_VERSION,
    "nqueens": LABEL_VERSION,
    "matmul": LABELS_MATMUL,
    "bh": LABELS_BH,
    "sort": LABELS_SORT
}

def print_stats(stats, avgs):
    """
    Runtime data presented in Table 1;
    Prints the (optimal) serial runtime, number of spawn operations,
    average number of steals for each benchmark and time for each spawn
    """

    print("-" * 100)
    print(" TABLE 1: RUNTIME METRICS")
    print("-" * 100)
    print(f"{'App':<8} {'Serial Runtime':>22} {'Nr. spawns':>15} {'Nr. steals':>15} {'Time per spawn (ms)':>25}")
    print("-" * 100)
    for app, avg in stats.items():
        seq_row = avgs[app].loc[avgs[app]['version'] == 0]
        if seq_row.empty:
            raise ValueError(f"No sequential version found for app {app}")
        sequential = seq_row['mean'].values[0]

        spawns = avg["spawns"]
        steals = avg["successful_steals"]
        time = avg["time_per_spawn"] * 1000

        print(f"{app:<8} {sequential:>20.2f} {spawns:>15.1e} {steals:>15.0f} {time:>25.4f}")
    print("-" * 100)


def print_workoverhead(overheads):
    """
    Data in Table 2.
    Print t_s, t_1 and the work overhead of each benchmark, parallelized with Velvet with and without the direct-recursion optimization
    """

    print("-" * 100)
    print(" TABLE 2: WORK OVERHEAD")
    print("-" * 100)
    print(f"{'App':<15} {'t_s':<15} {'t_1 (no opt)':<15} {'WO (no opt)':<15} {'t_1 (opt)':<15} {'WO (opt)':<15}")
    print("-" * 100)

    for row in overheads.itertuples(index=False):
        app, ts, seq_opt, seq_no, wo_opt, wo_no = row

        if app == 'tsp' or app == 'nqueens':
            tmp_seq = seq_no
            tmp_wo = wo_no
            seq_no = seq_opt
            wo_no = wo_opt
            seq_opt = tmp_seq
            wo_opt = tmp_wo
        elif app == 'sort-unsafe-app':
            continue
    
      
        print(f"{app:<15} {ts:<15.2f} {seq_no:<15.2f} {wo_no:<15.2f} {seq_opt:<15.2f} {wo_opt:<15.2f}")
    print("-" * 100)

         
def print_speedups(spdps):
    """
    Print the speedups of the different benchmarks.
    Data is presented in Fig. 4
    """
    header = f"{'Threads':<10} {'Version':<25} {'Speedup over sequential':<12}"

    print("-" * 70)
    print(" SPEEDUPS (Fig. 4)")
    for app, df in spdps.items():
        labels = LABELS[app]
        print("-" * 70)
        print(f"{'':<20} APP: {app} ")
        print(header)
        print("-" * 70)
        for thread in sorted(df["threads"].unique()):
            first = True
            for version, label in labels:
                row = df[(df["threads"] == thread) & (df["version"] == version)]
                if not row.empty:
                    r = row.iloc[0]
                    if first:
                        print(f"{thread:<10}{label:<25}{r['speedup']:<12.4f}")
                        first = False
                    else:
                        print(f"{'':<10}{label:<25}{r['speedup']:<12.4f}")
        print("-" * 70)


def print_c_comp(data):
    """
    Print the runtimes and speedup of the safe Velvet vs Cilk vs OpenMP benchmarks
    """
    header = f"{'Threads':<10} {'Version':<20} {'Mean':<12} {'Min':<12} {'Max':<12} {'Speedup':<12}"
    versions = [("Velvet", 2), ("Velvet, Unsafe App", 3), ("Cilk", 12), ("OpenMP", 11)]

    print("-" * 70)
    print(" RUNTIMES & SPEEDUP")
    for app, df in data.items():
        if app == "fib":
            continue
        print("-" * 75)
        print(f"{'':<20} APP: {app} ")
        print(header)
        print("-" * 70)
        for thread in sorted(df["threads"].unique()):
            first = True
            for version, idx in versions:
                row = df[(df["threads"] == thread) & (df["version"] == idx)]
                if not row.empty:
                    r = row.iloc[0]
                    if first:
                        print(f"{thread:<10}{version:<20}{r['mean']:<12.4f}{r['min']:<12.4f}{r['max']:<12.4f}{r['speedup']:<12.4f}")
                        first = False
                    else:
                        print(f"{'':<10}{version:<20}{r['mean']:<12.4f}{r['min']:<12.4f}{r['max']:<12.4f}{r['speedup']:<12.4f}")
        print("-" * 75)