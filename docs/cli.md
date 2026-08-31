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
| `--output-dir` | `results` | Results root (`<experiment>/<run_slug>/` is created under it) |

When `--seed` is omitted the seed is still materialized at start-up and recorded in both `parameters.seed` of `config.json` and `rng.master_seed` of `run.json`, so a run never proceeds on an unrecorded random number.

**Output files:**

```
results/                                          # the results root given by --output-dir
└── axelrod/                                      # experiment
    ├── latest_finished
    ├── simulate_20260415_120000_9f2c41ab_3b1d/   # one run (<subcommand>_<time>_<cfg8>_<exec4>)
    │   ├── run.json                              # run metadata (lineage / rng / research)
    │   ├── config.json                           # the condition lives under ["parameters"]
    │   ├── metrics.csv                           # run-scope metrics, long form
    │   ├── events.jsonl                          # one observation + one terminal line per trial
    │   ├── status.json / manifest.csv
    │   └── lock/                                 # copies of Cargo.lock / uv.lock
    └── figures/<run_slug>/                       # figures (outside the run: a figure is not part of its record)
```

runvault names the run directory. `--output-dir` is the results root, not the run itself. The most recent finished run is found with:

```bash
runvault path --experiment axelrod --latest --subcommand simulate
```

Trials (`--runs`) are not turned into child runs. They are the observed units of a single run, two lines each in `events.jsonl`: an `observation` at the moment the trial was looked at, and the `terminal` that ends it (`unit_id = trial-N`, `t` = its event count with `t_unit = event`, `budget` = `--max-events`, `censored = true` when it did not converge). A trial is looked at once, at its end, so the two lines share the same `t` — the numbers of the trial are only computed on the final board, so an intermediate line would carry no value that is not already derivable from `config.json` and the `terminal` line. runvault checks at write time that a censored line has `t == budget`, so a self-contradicting line never reaches the file. The run-level aggregate (convergence rate, mean region count, ...) is the `scope=run` part of `metrics.csv`.

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
| `--output-dir` | `results` | Results root (`<experiment>/<run_slug>/` is created under it) |

**Output files:**

A `sweep` produces one parent run plus one child run per (f, q) condition. The parent holds only the grid definition in `config.json` and writes no metrics (`rng.master_seed` is null). Each child is recorded as `subcommand = "simulate"`, in exactly the same shape as a standalone `simulate`, and points at the parent through `lineage.parent_run_uid` and `lineage.sweep_id`. Children sit beside the parent in the experiment directory, not underneath it.

```
results/axelrod/
├── sweep_20260415_120500_6d945b66_86c4/      # parent (grid definition)
├── simulate_20260415_120500_17ccc5bc_7076/   # child (f=5, q=5)
├── simulate_20260415_120501_8c542dd1_48f3/   # child (f=5, q=10)
└── ...
```

Per-condition results live in the children's `events.jsonl` / `metrics.csv`. For how to read the figures, see [Visualization](visualization.md).
