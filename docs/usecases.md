**English** | [日本語](usecases.ja.md)

# Use Cases

This project reimplements the culture-dissemination model from Axelrod (1997), "The Dissemination of Culture." Below are the typical things you can do with it, with pointers to the detailed documentation for each.

## 1. Reproduce the paper's benchmark (Table 7-2)

Reproduce the mean number of stable cultural regions on a 10×10 grid for the feature/trait combinations reported in Table 7-2, and confirm the characteristic two-dimensional asymmetry: more features lead to *fewer* regions (counter-intuitive), while more traits lead to *more* regions (intuitive).

- Run the base case: see [CLI — `simulate`](cli.md#simulate-single-parameter-set).
- Compare against the reported values and read the f×q interpretation: see [Reproduction](reproduction.md).

## 2. Parameter sensitivity (sweep)

Sweep the number of features $f$ and the number of traits $q$ over a grid to see how the equilibrium number of stable regions responds. The sweep makes the f×q asymmetry visible as a heatmap.

- Run a grid search: see [CLI — `sweep`](cli.md#sweep-parameter-sweep).
- Visualize heatmaps and marginal line plots: see [Visualization — sweep](visualization.md#sweep-visualization).

## 3. Visualize and interpret results

Turn a `simulate` or `sweep` result into figures and read off the qualitative behavior: convergence to a single culture versus the survival of multiple stable regions (global cultural polarization).

- Generate figures from the most recent run: see [Visualization](visualization.md).
- Read the `metrics.csv` column reference and "how to read typical results": see [Visualization — output interpretation](visualization.md#output-interpretation).

## Where to go next

- [CLI](cli.md) — every Rust CLI subcommand and option.
- [Reproduction](reproduction.md) — the Table 7-2 benchmark and how to reproduce it.
- [Visualization](visualization.md) — the Python `analysis/visualize.py` and how to read the outputs.
- [Architecture](architecture.md) — repository structure and how the model is built on the socsim framework.
