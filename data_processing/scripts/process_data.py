# LOAD DATA, PROCESS AVERAGES
import pandas as pd
from pathlib import Path

def load_data(filename, column_names):
    """
    Load raw runtime data from CSV into a Pandas DataFrame.

    Parameters
    ----------
    filename : str
        Path to input CSV file
    column_names : list[str]
        Column names to extract from CSV

    Returns
    -------
    pd.DataFrame
        DataFrame containing only the requested columns.
    """
    df = pd.read_csv(filename, usecols=column_names)
    return df

def compute_average_runtimes(data):
    """
    Compute mean, min and max runtimes excluding first (warmup) run

    Parameters
    ----------
    data : pd.DataFrame
        raw data with columns [version, threads, time]
    
    Returns
    -------
    averages : pd.DataFrame
        Structured array with fields:
        ('version', int), 
        ('threads', int),
        ('mean', float),
        ('min', float),
        ('max', float)
    """
    data.columns = ["version", "threads", "time"] # standardise naming!

    averages = []
    for (version, threads), group in data.groupby(["version", "threads"]):
        times = group["time"].values
        if len(times) > 1: # need at least 2 to skip the first
            vals = times[1:]
        
            averages.append({
                "version": version,
                "threads": threads,
                "mean": vals.mean(),
                "min": vals.min(),
                "max": vals.max(),
            })
    return pd.DataFrame(averages)

def prepare_averages(file, cols, app):
    """
    Load raw data and compute averages

    Parameters
    ----------
    file :  str
        Filepath to dataset
    cols : list[str]
        Column names to extract
    app : str
        Corresponding application key; COMPILE, BF
    
    Returns
    -------
    dict with:
      'app' : str
        app or 'compiletimes' or 'bf'
      'avgs' : pd.DataFrame
        averages DataFrame
    """
    raw = load_data(file, cols)
    avgs = compute_average_runtimes(raw)

    return {
        "app": app,
        "avgs": avgs,
    }

def compute_speedup(averages, skip=None, only=None, seq=0):
    """
    Compute speedup of parallel run relative to sequential baseline

    Parameters
    ----------
    averages : pd.DataFrame
        Structured array with fields:
        ('version', int), 
        ('threads', int),
        ('mean', float),
        ('min', float),
        ('max', float)
    skip : optional int
        versions to skip
    incl : optional int
        versions to include (Exclusively)
    seq : int
        version-number of sequential run, to use for speedup computation
    
    Returns
    -------
    speedup : pd.DataFrame
        Structured array with fields:
        ('version', int), 
        ('threads', int),
        ('speedup', float),
    """
    seq_row = averages.loc[averages['version'] == seq]
    if seq_row.empty:
        raise ValueError(f"No sequential version {seq} found")
    sequential = seq_row['mean'].values[0]

    # filter versions
    if only is not None:
        df_filtered = averages[averages['version'].isin(only)]
    elif skip is not None:
        df_filtered = averages[~averages['version'].isin(skip)]
    else:
        df_filtered = averages

    speedup_df = pd.DataFrame({
        'version': df_filtered['version'],
        'threads': df_filtered['threads'],
        'speedup': sequential / df_filtered['mean']
    })

    return speedup_df

def prepare_stats(data_dir):
    """
    Load raw data with runtime stats (of a single application) and compute average number of spawns and steals.
    Collects all CSV files in the given directory, loads the relevant columns and collects the per-file sum for each metric.
    Double checks that num stolen == num successful steals and that number of spawns is consistent across files

    Parameters
    ----------
    data_dir :  str
        Filepath to directory holding datasets
    
    Returns
    -------
    avg_sums: pd.DataFrame
        DataFrame with averages columns ['successful_steals', 'spawns', 'total_stolen_jobs']
    """
    sums_per_file = {}

    for path in Path(data_dir).glob("*.csv"):
        df = pd.read_csv(path, usecols=['successful_steals', 'spawns', 'total_stolen_jobs'])

        # per-file sums
        sums = df.sum()
        sums_per_file[path.name] = sums

        # sanity check: successful steals == stolen jobs
        if sums["successful_steals"] != sums["total_stolen_jobs"]:
            print(f"Mismatch in {path.name}: sum(successful_steals)={sums['successful_steals']}, sum(total_stolen_jobs)={sums['total_stolen_jobs']}")
        
    # sanity check: number of spawns is consistent across files
    spawns = [s["spawns"] for s in sums_per_file.values()]
    if len(set(spawns)) != 1 and not data_dir.endswith("tsp/"):
        print(f"Inconsistent number of spawns across files in directory: {data_dir}")
        for fname, sums in sums_per_file.items():
            print(f"  {fname}:spawns={sums['spawns']}")

    all_sums = pd.DataFrame.from_dict(sums_per_file, orient="index")
    avg_sums = all_sums.mean()

    return avg_sums

def add_time_per_spawn(avgs, stats):
    """
    Figure out the average 'time per spawn' by dividing the single-threaded Velvet runtime with the number of spawns

    Parameters
    ----------
    avgs : dict
        Each entry has:
          'app' : str - application name
          'avgs' : pd.DataFrame - columns ['version', 'threads', 'mean']
    
    stats : dict
        Each entry has:
          'app' : str - application name
          'stats' : pd.DataFrame - columns including 'spawns'
          Modified in-place to add a new column 'time_per_spawn'
    """
    for app, stats_df in stats.items():
        runtimes = avgs[app]
        # single-threaded runtime for Velvet with the SAFE QUEUE (version==2, threads==1)
        rt = runtimes.loc[(runtimes['version'] == 2) & (runtimes['threads'] == 1), 'mean']
        if rt.empty:
            raise ValueError(f"No single-threaded Velvet runtime found for app {app}")
        rt = rt.iloc[0]

        # Compute time per spawn for each row in stats
        if 'spawns' not in stats_df:
            raise ValueError(f"'spawns' column missing in stats for app {app}")
        
        stats_df['time_per_spawn'] = rt / stats_df['spawns']

        # Update the stats dictionary in-place
        stats[app] = stats_df

def compute_wo(avgs):
    """
    Extracts t_s and t_1 for each benchmark, including the unsafe Sort application, and computes the work overhead.
    Does so for each queue backend, and ensures correct t_s is used.

    Parameters
    ----------
    avgs : dict
        Each entry has:
          'app' : str - application name
          'avgs' : pd.DataFrame - columns ['version', 'threads', 'mean']
    
    Returns
    -------
    work_overheads: pd.DataFrame
        DataFrame with rows ['app', 'ts', 'safe', 'wo_safe', '..]
    """
    work_overheads = []
    # fib - no threshold
    fib_df = avgs["fib"]
    single_threaded = fib_df[fib_df["threads"] == 1]
    row = {"app": "fib-no-thresh"}
    for version, entry in [(0, "ts"), (3, "velvet"), (8, "recursion")]:
        val = single_threaded.loc[single_threaded["version"] == version, "mean"]
        row[f"{entry}"] = val.iloc[0] if not val.empty else None
    work_overheads.append(row)

    for app, df in avgs.items():
        single_threaded = df[df["threads"] == 1]
        if app in ["adapint", "fib", "tsp"]:
            ts = 0
        else:
            ts = -1
        
        row = {"app": app}
        for version, entry in [(ts, "ts"), (2, "velvet"), (7, "recursion")]:
            val = single_threaded.loc[single_threaded["version"] == version, "mean"]
            row[f"{entry}"] = val.iloc[0] if not val.empty else None
        work_overheads.append(row)
        

    # sort - unsafe app
    sort_df = avgs["sort"]
    single_threaded = sort_df[sort_df["threads"] == 1]
    row = {"app": "sort-unsafe-app"}
    for version, entry in [(-2, "ts"), (3, "velvet"), (8, "recursion")]:
        val = single_threaded.loc[single_threaded["version"] == version, "mean"]
        row[f"{entry}"] = val.iloc[0] if not val.empty else None
    work_overheads.append(row)

    df = pd.DataFrame(work_overheads)

    for version in ["velvet", "recursion"]:
        df[f"wo_{version}"] = df[version] / df["ts"]
    
    return df