import os
from process_data import prepare_averages, compute_speedup, prepare_stats, add_time_per_spawn, compute_wo
from print import print_workoverhead, print_stats, print_speedups, print_c_comp
from plot import plot_c_comp, plot_speedup, SHOW, SAVE

# Table 1. Runtime metrics
def stats(avgs, paths):
    stats = {}
    for path, app in paths:
        stat = prepare_stats(path)
        stats[app] = stat
    add_time_per_spawn(avgs, stats)
    print_stats(stats, avgs)

# Table 2. Work overhead (WO) for Velvet with and without the direct-recursion optimization
def work_overhead(avgs):
    overheads = compute_wo(avgs)
    print_workoverhead(overheads)

# Fig. 1. Parallel speedup of six benchmarks, comapred with Rayon
def rayon_comparison(avgs, filename, mode=SHOW):
    spdps = {}
    for app, avg in avgs.items():
        if app == "fib":
            continue
        spdp = compute_speedup(avg, skip=[10,11])
        spdps[app] = spdp
    plot_speedup(spdps, mode, filename)
    # print_speedups(spdps)

# Fig. 2. Parallel runtime and speedup of six benchmarks, comapred with C-frameworks
def c_comparison(avgs, filename, mode=SHOW):
    import pandas as pd
    avg_spdp = {}
    for app, avg in avgs.items():
        if app == "fib":
            continue
        seq_r_row = avg.loc[avg['version'] == 0]
        if seq_r_row.empty:
            raise ValueError(f"No sequential version found for {app}")
        seq_r = seq_r_row['mean'].values[0]

        seq_c_row = avg.loc[avg['version'] == 10]
        if seq_c_row.empty:
            raise ValueError(f"No sequential version found for {app}")
        seq_c = seq_c_row['mean'].values[0]
        seq = 10 if seq_c < seq_r else 0

        if app == 'matmul':
            seq_unsafe_row = avg.loc[avg['version'] == -3]
            if seq_unsafe_row.empty:
                raise ValueError(f"No sequential matmul unsafe version found")
            seq_unsafe = seq_unsafe_row['mean'].values[0]
            if seq_unsafe < seq:
                seq = -3

        # speedup has to be relative to the fastest sequential overall!
        spdp_rust = compute_speedup(avg, skip=[11,12], seq=seq)
        spdp_c = compute_speedup(avg, only=[11,12], seq=seq)
        all_speedups = pd.concat([spdp_rust, spdp_c])
        avg_spdp[app] = avg.merge(
            all_speedups[['version', 'threads', 'speedup']], 
            on=['version', 'threads'], 
            how='left'
        )
    
    # print_c_comp(avg_spdp)
    plot_c_comp(avg_spdp, mode, filename)

import argparse
def main():
    parser = argparse.ArgumentParser(
        description="Generate figures and tables from Velvet paper, using benchmark data."
    )
    
    parser.add_argument(
        "action",
        choices=["all", "rust-only", "tab1", "tab2", "fig1", "fig2", "rayon-comp","c-comp"],
        help="Which table or figure to generate."
    )

    parser.add_argument('path', help="(Relative) path to directory holding all data files")
    
    mode_group = parser.add_mutually_exclusive_group()
    mode_group.add_argument("--show", action="store_true", help="Show plots interactively.")
    mode_group.add_argument("--save", action="store_true", help="Save plots as PDFs to figs/")

    args = parser.parse_args()
    metrics = ['version', 'num_workers', 'time_secs']
    apps = {
        'fib':     metrics,
        'adapint': metrics,
        'tsp':     metrics,
        'nqueens': metrics,
        'bh':      ['version', 'num_workers', 'forces_time'],
        'matmul':  metrics,
        'sort':    metrics,
    }
    data_files = [(f'{args.path}/{app}.csv', app, m) for app, m in apps.items()]

    stats_paths = [(f'{args.path}/stats/{app}/', app) for app, _ in apps.items()]

    script_dir = os.path.dirname(os.path.abspath(__file__))
    filename1 = os.path.join(script_dir, '../figs/fig1_vs_rayon.pdf')
    filename2 = os.path.join(script_dir, '../figs/fig2_vs_c.pdf')
    avgs = {}
    for file, app, cols in data_files:
        avg = prepare_averages(file, cols, app)
        avgs[avg["app"]] = avg["avgs"]

    if args.show:
        mode=SHOW
    else:
        mode=SAVE
    if args.action == "all":
        stats(avgs, stats_paths)
        work_overhead(avgs)
        rayon_comparison(avgs, filename1, mode=mode)
        c_comparison(avgs, filename2, mode=mode)
    elif args.action == "rust-only":
        stats(avgs, stats_paths)
        work_overhead(avgs)
        rayon_comparison(avgs, filename1, mode=mode)
    elif args.action == "tab1":
        stats(avgs, stats_paths)
    elif args.action == "tab2":
        work_overhead(avgs)
    elif args.action == "rayon-comp" or args.action == "fig1":
        rayon_comparison(avgs, filename1, mode=mode)
    elif args.action == "c-comp" or args.action == "fig2":
        c_comparison(avgs, filename2, mode=mode)

if __name__ == "__main__":
    main()
