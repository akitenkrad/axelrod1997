**English** | [日本語](cli.ja.md)

# Rust CLI

The `axelrod-culture` crate exposes a CLI (binary `axelrod`) with the subcommands `simulate` and `sweep`. Build it once with `cargo build --release`; run from the workspace root with `cargo run --release -- <subcommand> ...`.

## The model in one paragraph

Each site on a `width × height` grid carries a culture vector of length `f` (the number of features), whose entries are integers in `0..q` (the number of traits). One event picks an active site $s$ uniformly at random, picks a neighbor $nb$ uniformly at random from $s$'s von Neumann neighborhood (non-toroidal, 4-neighbor), computes the similarity $\mathrm{sim} = (\text{matching features}) / f$, and with probability $\mathrm{sim}$ interacts: a feature on which they differ is chosen at random and $s$ copies $nb$'s value. When $\mathrm{sim} \in \{0, 1\}$ no interaction occurs. The board is stable when $\mathrm{sim} \in \{0, 1\}$ for every adjacent pair, and the main metric is the number of stable cultural regions (the number of 4-connected components of identical culture vectors).

## `simulate` (single parameter set)

Run a single feature/trait setting over a number of runs.

```bash
# Build
cargo build --release

# Run the integration tests (verify similarity / is_stable / count_stable_regions /
# random_init / a small e2e convergence — tests/integration_test.rs)
cargo test --release

# Run a single parameter set (Table 7-2 base case: f=5, q=10, 10×10, 10 runs)
cargo run --release -- simulate \
    --features 5 --traits 10 \
    --runs 10 --seed 42
```

**Options for the `simulate` subcommand:**

| Option | Default | Description |
|--------|---------|-------------|
| `--width` | 10 | Grid width |
| `--height` | 10 | Grid height |
| `--features` / `-f` | 5 | Number of features $f$ |
| `--traits` / `-q` | 10 | Number of traits $q$ |
| `--runs` | 10 | Number of runs |
| `--max-events` | 1000000 | Maximum number of events per run |
| `--seed` | — | Random seed (base value; random if omitted) |
| `--output-dir` | `results` | Output directory |

**Output files:**

```
results/
├── latest -> simulate_20260415_120000        # symlink to the most recent run
├── simulate_20260415_120000/
│   ├── config.json                           # run-time configuration (subcommand="simulate")
│   └── metrics.csv                           # metrics per run
└── ...
```

## `sweep` (parameter sweep)

Grid-search over the number of features $f$ and the number of traits $q$.

```bash
# Reproduce all Table 7-2 conditions (3×3) with 10 runs each
cargo run --release -- sweep \
    --features-min 5 --features-max 15 --features-step 5 \
    --traits-min   5 --traits-max   15 --traits-step   5 \
    --runs 10 --seed 42
```

**Options for the `sweep` subcommand:**

| Option | Default | Description |
|--------|---------|-------------|
| `--width` | 10 | Grid width |
| `--height` | 10 | Grid height |
| `--features-min` | 5 | Start value of features $f$ |
| `--features-max` | 15 | End value of features $f$ (inclusive) |
| `--features-step` | 5 | Step of features $f$ |
| `--traits-min` | 5 | Start value of traits $q$ |
| `--traits-max` | 15 | End value of traits $q$ (inclusive) |
| `--traits-step` | 5 | Step of traits $q$ |
| `--runs` | 10 | Number of runs per condition |
| `--max-events` | 1000000 | Maximum number of events per run |
| `--seed` | — | Random seed (base value) |
| `--output-dir` | `results` | Base output directory |

**Output files:**

```
results/
├── latest -> sweep_20260415_120500
├── sweep_20260415_120500/
│   ├── config.json                           # run-time configuration (subcommand="sweep")
│   └── metrics.csv                           # results for every (f, q, run)
└── ...
```

`simulate` and `sweep` use a symmetric directory naming scheme (`simulate_<ts>` / `sweep_<ts>`), and both write their configuration to `config.json` and their results to `metrics.csv`.

For the `metrics.csv` column reference and how to read the figures, see [Visualization](visualization.md).
