**English** | [日本語](visualization.ja.md)

# Visualization (Python)

The Python tooling is a single unified script, `analysis/visualize.py`. Python dependencies are managed with [uv](https://docs.astral.sh/uv/).

```bash
# Install dependencies
uv sync

# Visualize the result (auto-references results/latest)
# visualize.py is a unified script supporting both simulate and sweep.
# It auto-detects the mode from the subcommand field in config.json (or, if
# absent, from the simulate_ / sweep_ prefix of the directory name).
uv run python analysis/visualize.py

# Visualize a specific result (same command for simulate or sweep)
uv run python analysis/visualize.py --results_dir results/simulate_20260415_120000
uv run python analysis/visualize.py --results_dir results/sweep_20260415_120500
```

**Output files (simulate):**

```
results/latest/figures/
├── simulate_distribution.png   # distribution of the stable-region count (box + jitter)
├── simulate_metrics.png        # mean ±95% CI per metric
└── simulate_vs_table7_2.png    # comparison against the Table 7-2 benchmark (when applicable)
```

## Sweep visualization

When the result is a `sweep`, the same command produces the f×q sweep figures.

**Output files (sweep):**

```
results/latest/figures/
├── sweep_heatmap_regions.png   # f×q heatmap of mean n_stable_regions
├── sweep_heatmap_ci.png        # f×q heatmap of the 95% CI
├── sweep_marginal_features.png # marginal line plot with f on the X axis (one line per q)
├── sweep_marginal_traits.png   # marginal line plot with q on the X axis (one line per f)
└── sweep_overview.png          # 2×2 overview panel
```

## Output interpretation

### metrics.csv columns (common to simulate / sweep)

| Column | Description |
|--------|-------------|
| `run` | Run index (0-indexed) |
| `width`, `height` | Grid size |
| `features`, `traits` | Number of features $f$ / number of traits $q$ |
| `seed` | The derived seed used for that run |
| `converged` | Whether a stable state was reached (true/false) |
| `n_events` | Number of events actually executed |
| `n_stable_regions` | Number of stable cultural regions (4-connected components). **The main metric** |
| `max_region_size` | Size of the largest region (number of sites) |
| `n_distinct_cultures` | Number of distinct culture vectors present on the board |

### How to read typical results

- **Large $f$, small $q$** → regions converge to 1, yielding a single culture.
- **Small $f$, large $q$** ($f=5, q=15$) → many stable regions survive, and global cultural polarization emerges endogenously.
- **Convergence check**: when `converged=true`, every adjacent pair satisfies $\mathrm{sim} \in \{0, 1\}$. When `false`, increase `--max-events` and re-run.
