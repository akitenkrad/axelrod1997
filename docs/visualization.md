**English** | [日本語](visualization.ja.md)

# Visualization (Python)

The Python tooling is a single unified script, `analysis/visualize.py`. Python dependencies are managed with [uv](https://docs.astral.sh/uv/).

```bash
# Install dependencies
uv sync

# Visualize the most recent simulate run
# runvault picks the run:
#   runvault path --experiment axelrod --latest --subcommand simulate --standalone
# --standalone is the default: a sweep child shares its subcommand with a run started
# by hand, so without it you get whichever child ran last.
# The mode comes from the subcommand field of run.json, never from the directory name.
uv run python analysis/visualize.py

# To let sweep children be candidates as well
uv run python analysis/visualize.py --include-sweep-children

# Visualize the most recent sweep (the parent run)
# Without --subcommand, --latest can hand you the sweep parent, so always pass it.
uv run python analysis/visualize.py --subcommand sweep

# When the results root is not the default (results)
uv run python analysis/visualize.py --results_root /tmp/rv-axelrod

# Point at a run directory directly (same command for simulate or sweep)
uv run python analysis/visualize.py --results_dir results/axelrod/simulate_20260415_120000_9f2c41ab_3b1d
```

If `runvault` is not on `PATH`, point at the binary with the `RUNVAULT` environment variable.

```bash
RUNVAULT=~/Documents/workspace/rust/rs-runvault/target/debug/runvault \
    uv run python analysis/visualize.py
```

**Output files (simulate):**

Figures land *outside* the run directory. `manifest.csv` is settled by `finish()`, so a figure written into the run afterwards would contradict its own record.

```
results/axelrod/figures/<run_slug>/
├── simulate_distribution.png   # distribution of the stable-region count (box + jitter)
├── simulate_metrics.png        # mean ±95% CI per metric
└── simulate_vs_table7_2.png    # comparison against the Table 7-2 benchmark (when applicable)
```

## Sweep visualization

When the result is a `sweep`, the same command produces the f×q sweep figures.

**Output files (sweep):**

```
results/axelrod/figures/<run_slug>/
├── sweep_heatmap_regions.png   # f×q heatmap of mean n_stable_regions
├── sweep_heatmap_ci.png        # f×q heatmap of the 95% CI
├── sweep_marginal_features.png # marginal line plot with f on the X axis (one line per q)
├── sweep_marginal_traits.png   # marginal line plot with q on the X axis (one line per f)
└── sweep_overview.png          # 2×2 overview panel
```

## Output interpretation

### One line of events.jsonl (one trial; `schema` is `terminal`)

| Field | Description |
|--------|-------------|
| `unit_id` | The observed unit: `trial-N` (N is the 0-indexed trial number) |
| `t` | Number of events at which the trial ended. **Reserved** |
| `t_unit` | Unit of `t`: `event` (the model's tick in Axelrod is a single event). **Reserved** |
| `outcome` | `converged` / `unconverged`. **Reserved** |
| `censored` | Whether the trial hit the ceiling without converging; if true, `t == budget`. **Reserved** |
| `budget` | The event ceiling (`--max-events`). **Reserved** |
| `seed` | The derived seed used for that trial |
| `n_stable_regions` | Number of stable cultural regions (4-connected components). **The main metric** |
| `max_region_size` | Size of the largest region (number of sites) |
| `n_distinct_cultures` | Number of distinct culture vectors present on the board |

Grid size, $f$ and $q$ do not vary between trials, so they live in `parameters` of `config.json` rather than on every event line.

### Rows of metrics.csv (the run-level aggregate; `scope=run`)

| Metric | Description |
|--------|-------------|
| `n_units` | Number of trials. **A runvault reserved metric name** |
| `n_converged` | Number of trials that converged |
| `convergence_rate` | Fraction that converged |
| `mean_n_stable_regions` | Mean number of stable regions. **The main metric** |
| `mean_max_region_size` | Mean size of the largest region |
| `mean_n_distinct_cultures` | Mean number of distinct cultures |
| `mean_n_events` | Mean number of events executed |

### How to read typical results

- **Large $f$, small $q$** → regions converge to 1, yielding a single culture.
- **Small $f$, large $q$** ($f=5, q=15$) → many stable regions survive, and global cultural polarization emerges endogenously.
- **Convergence check**: when `outcome=converged`, every adjacent pair satisfies $\mathrm{sim} \in \{0, 1\}$. When `unconverged` (i.e. `censored=true`), increase `--max-events` and re-run.
