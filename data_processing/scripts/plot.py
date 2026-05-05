import numpy as np
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

SHOW=0
SAVE=1

plt.rcParams['font.family'] = 'serif'
plt.rcParams['font.serif'] = ['Times New Roman'] + plt.rcParams['font.serif']
plt.rcParams['axes.labelsize'] = 14
plt.rcParams['legend.fontsize'] = 12
plt.rcParams['axes.titlesize'] = 16

# plt.rcParams.update({'font.size': plt.rcParams['font.size'] + 4})
plt.rc('text', usetex=True)
plt.rcParams['text.latex.preamble'] = r'\boldmath'

blues = ["#92c5f2", "#5699d6", "#1f77b4"]
oranges = ['#ff7f0e', "#943611",]
colours = ['tab:green', 'tab:red', 'tab:olive', 'tab:purple', 'tab:pink', 'tab:gray']

LABEL_VERSION= [(2,"Velvet",('o', oranges[0])), (1,"Rayon",('^', blues[2]))]
LABELS_MATMUL = [(2,"Velvet",('o', oranges[0])), (1,"Rayon, Ours",('^', blues[2])),  (5,"Rayon, Demo-Z",('^',blues[1])), (6,"Rayon, Demo-Strassen",('^', blues[0]))]
LABELS_BH = [(2,"Velvet",('o', oranges[0])), (5,"Rayon, ChildIter",('^',blues[2])), (1,"Rayon, TreeIter",('^', blues[1])), (6,"Rayon, BodyIter",('^',blues[0]))]
LABELS_SORT = [(2,"Velvet",('o', oranges[0])), (1,"Rayon",('^', blues[2])), (3, "Velvet, Unsafe Application",('o', oranges[1]))]

LABEL_C_COMP = [(r'\textbf{Velvet}', 2, ('o', oranges[0])), (r'\textbf{Cilk}', 12, ('^', colours[0])), (r'\textbf{OpenMP}', 11, ('s', colours[1]))]
LABEL_C_COMP_UNSAFE =[(r'\textbf{Velvet}', 2, ('o', oranges[0])), (r'\textbf{Velvet, Unsafe Application}', 3, ('o', oranges[1])), (r'\textbf{Cilk}', 12, ('^', colours[0])), (r'\textbf{OpenMP}', 11, ('s', colours[1]))]

APP_NAMES=[r'\textbf{Fibonacci (\textit{fib})}',
           r'\textbf{Adaptive Integration (\textit{adapint})}', 
           r'\textbf{Traveling Salesperson (\textit{tsp})}', 
           r'\textbf{N-Queens (\textit{nqueen})}', 
           r'\textbf{Barnes-Hut (\textit{bh})}', 
           r'\textbf{Matrix Multiplication (\textit{matmul})}', 
           r'\textbf{Merge Sort (\textit{sort})}']

SETTINGS = {
    "adapint": (1, LABEL_VERSION, LABEL_C_COMP),
    "tsp": (2, LABEL_VERSION, LABEL_C_COMP),
    "nqueens": (3, LABEL_VERSION, LABEL_C_COMP),
    "matmul": (5, LABELS_MATMUL, LABEL_C_COMP_UNSAFE),
    "bh": (4, LABELS_BH, LABEL_C_COMP),
    "sort": (6, LABELS_SORT, LABEL_C_COMP_UNSAFE)
}

#################################################
####                FIGURE 1                 ####
#################################################
def plot_speedup(data_to_plot, mode, filename="../figs/speedup.pdf"):
    """
    Generate Fig.1.
    Six speedup graphs, comparing Velvet with Rayon
    """    
    fig, axs = plt.subplots(2, 3, figsize=(16,10))
    axs = axs.flatten()
    
    for i, entry in enumerate(data_to_plot.items()):
        ax = axs[i]
        plot_speedup_ax(entry, ax)
        if i%3 != 0:
            ax.set_ylabel('') # remove y-axis label
            ax.tick_params(labelleft=False) # hide y-axis tick labels
    
    # plt.tight_layout(rect=[0, 0.05, 1, 1], h_pad=3.0)
    fig.subplots_adjust(
        left=0.05,
        right=0.99,
        top=0.96,
        bottom=0.06,
        wspace=0.02,   # horizontal space between subplots
        hspace=0.30    # vertical space between subplots
    )
    if mode == SHOW:
        print(f"\nFIGURE 1: VELVET VS RAYON ")
        plt.show()
    else:
        plt.savefig(filename)
        print(f"\nFIGURE 1: VELVET VS RAYON saved to {filename}")

def plot_speedup_ax(data, ax):
    """
    Speedup graph for a benchmark

    Parameters
    ----------
    data : (key: str, pd.DataFrame: df)
        df columns: ['version', 'threads', 'speedup']
    ax : matplotlib.axes.Axes
        Axes to plot on.
    """
    (app, df) = data
    (title_id, labels, _) = SETTINGS[app]
    
    for version, label, markers in labels:
        group = df[df["version"] == version].sort_values("threads")
        
        (marker, colour) = markers
        ax.plot(
            group["threads"],
            group["speedup"],
            label = fr'\textbf{{{label}}}',
            marker=marker,
            c=colour
        )

    tmin, tmax = df["threads"].min(), df["threads"].max()
    ax.plot(
        [tmin, tmax],
        [tmin, tmax],
        label=r'\textbf{Linear}',
        linestyle='--',
        color='black'
    )

    ax.set_xlim(left=0)
    ax.set_ylim(bottom=0)
    if tmax < 50:
        ax.set_xticks(np.arange(0, tmax+1, 4))
        ax.set_yticks(np.arange(0, tmax+1, 4))
    else:
        ax.set_xticks(np.arange(0, tmax+1, 16))
        ax.set_yticks(np.arange(0, tmax+1, 16))

    ax.set_title(APP_NAMES[title_id])
    ax.set_ylabel(r'\textbf{Speedup over serial}')
    ax.set_xlabel(r'\textbf{Threads}')

    ax.grid(axis='y', which='major', linestyle='--', alpha=0.6, zorder=0)
    ax.grid(axis='y', which='minor', linestyle=':', alpha=0.5, zorder=0)

    ax.grid(axis='both', linestyle='--', alpha=0.6, zorder=0)
    ax.legend(frameon=False,loc='upper left')


#################################################
####               FIGURE 2                  ####
#################################################
def plot_c_comp(data_to_plot, mode, filename="../figs/framework_runtimes.pdf"):
    """
    Generate Fig.2.
    Six graphs, showing both bar plots with runtimes and speedup lines.
    """

    fig, axs = plt.subplots(2, 3, figsize=(14,8))
    axs = axs.flatten()

    for i, entry in enumerate(data_to_plot.items()):
        plot_runtimes_w_speedups(entry, axs[i], i)

        if i%3 != 0:
            axs[i].set_ylabel('') # remove y-axis label

    # combine all legends into one - subplot 4 has the most 'extra' labels for the legend so using that one
    handles, labels = axs[4].get_legend_handles_labels()
    fig.legend(handles, labels, loc='lower center', ncol=7, frameon=False)

    fig.subplots_adjust(
        left=0.05,
        right=0.95,
        top=0.95,
        bottom=0.12,
        wspace=0.15,   # horizontal space between subplots
        hspace=0.30    # vertical space between subplots
    )

    if mode == SHOW:
        print(f"\nFIGURE 2: VELVET VS C")
        plt.show()
    else:
        plt.savefig(filename)
        print(f"\nFIGURE 2: VELVET VS C saved to {filename}\n")

def plot_runtimes_w_speedups(data, ax, ax_nr):
    (app, df) = data
    (title_id, _, datalist) = SETTINGS[app]

    ax_speedup = ax.twinx()

    bar_width = 0.8 / 4
    thread_counts = sorted(df["threads"].unique())
    x = np.arange(len(thread_counts) + 1)  # +1 for "0 threads"
    ymax = 0

    # sequentials
    if app == 'matmul':
        serial_data = [(r'\textbf{Serial Rust}', 0, colours[2]), (r'\textbf{Serial Unsafe Rust}', -3, colours[4]), (r'\textbf{Serial C}', 10, colours[3])]
    else:
        serial_data = [(r'\textbf{Serial Rust}', 0, colours[2]), (r'\textbf{Serial C}', 10, colours[3])]
    for i, (version, v, c) in enumerate(serial_data):
        group = df[(df["version"] == v)]
        if group.empty: continue
        
        mean = group["mean"].values
        lo_err = (group["mean"] - group["min"]).values
        hi_err = (group["max"] - group["mean"]).values

        ax.bar(
            x[0] + i * bar_width,
            mean,
            yerr=[lo_err, hi_err],
            width=bar_width,
            label=version,
            zorder=3,
            capsize=1,
            error_kw={'elinewidth':1, 'alpha':0.8},
            alpha=0.8,
            color=c,
        )

        ymax = max(ymax, mean + hi_err)

    for i, (version, v, (m, c)) in enumerate(datalist):
        group = df[df["version"] == v].sort_values("threads")
        means = group["mean"].values
        lo_err = (group["mean"] - group["min"]).values
        hi_err = (group["max"] - group["mean"]).values
        speedups = group["speedup"].values
    
        ax.bar(
            x[1:] + i * bar_width, 
            means,
            yerr=[lo_err, hi_err],
            width=bar_width,
            label=version, 
            zorder=3,
            capsize=1,
            error_kw={'elinewidth':1, 'alpha':0.8},
            alpha=0.8,
            color=c,
        )
        ymax = max(ymax, np.max(means + hi_err))

        ax_speedup.plot(
            x[1:] + bar_width,
            speedups, 
            marker=m, 
            color=c, 
            linewidth=1.2,
            markersize=3.5,
            markeredgewidth=0.5,
            label=version,
        )

    # --- Y-axis scaling ---
    if ymax < 25:
        step = 2
    elif ymax < 100:
        step = 5
    else:
        step = 10
    top = np.floor((ymax + step) / step) * step
    ax.set_ylim(0, top+1)
    ax.set_ylabel(r'\textbf{Runtime (seconds)}')

    tmax = thread_counts[-1]
    ax_speedup.set_ylim(0, tmax+1)
    ax_speedup.plot(
        x[1:] + bar_width,   # Center the line on the middle bar of each group
        thread_counts,       # Speedup value = thread count
        label=r'\textbf{Linear}', 
        linestyle='--', 
        color='black', 
        alpha=0.7,
        zorder=4
    )
    if tmax < 50:
        ax_speedup.set_yticks(np.arange(0, tmax+1, 4))
    else:
        ax_speedup.set_yticks(np.arange(0, tmax+1, 16))

    if ax_nr%3 != 2:
        ax_speedup.tick_params(axis='y', which='both', labelright=False, right=True) # hide y-axis tick labels
    else:
        ax_speedup.set_ylabel(r'\textbf{Speedup over fastest serial}')

    
    # --- X-axis (categorical placement) ---
    unique_threads = df["threads"].unique()
    unique_threads = np.insert(unique_threads, 0, 1)
    ax.set_xticks(x + bar_width)
    ax.set_xticklabels(unique_threads.astype(int))
    ax.set_xlabel(r'\textbf{Threads}')

    # --- Styling ---
    ax.set_title(APP_NAMES[title_id])
    ax.yaxis.set_major_locator(mticker.MultipleLocator(step))
    ax.yaxis.set_minor_locator(mticker.AutoMinorLocator(2))
    ax.grid(axis='y', which='major', linestyle='--', alpha=0.6, zorder=0)
    ax.grid(axis='y', which='minor', linestyle=':', alpha=0.5, zorder=0)
