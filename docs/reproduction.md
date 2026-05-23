**English** | [日本語](reproduction.ja.md)

# Paper Reproduction (Table 7-2)

The central quantitative result of Axelrod (1997) is Table 7-2: the mean number of stable cultural regions on a 10×10 grid, averaged over 10 runs per condition, for several combinations of the number of features $f$ and the number of traits $q$.

## The benchmark (Table 7-2)

10×10 grid; mean number of stable cultural regions over 10 runs per condition.

| Features $f$ | Traits $q$ | Paper value | Reproduction target |
|--------------|------------|-------------|---------------------|
| 5 | 5 | 1.0 | ±0.3 |
| 5 | 10 | 3.2 | ±0.5 |
| 5 | 15 | 20.0 | ±3.0 |
| 10 | 5 | 1.0 | ±0.3 |
| 10 | 10 | 1.0 | ±0.3 |
| 10 | 15 | 1.4 | ±0.3 |
| 15 | 5 | 1.0 | ±0.3 |
| 15 | 10 | 1.0 | ±0.3 |
| 15 | 15 | 1.2 | ±0.3 |

## The two-dimensional asymmetry

The table reveals an asymmetric effect of the two parameters:

- **Increasing $f$ (the number of features) → fewer regions** (counter-intuitive). The more features two sites share, the higher their similarity, so they converge more readily.
- **Increasing $q$ (the number of traits) → more regions** (intuitive). Higher initial diversity makes it harder for sites to find common ground.

The dramatic case is $f=5, q=15$: many stable regions survive, and global cultural polarization emerges endogenously rather than collapsing to a single shared culture.

## How to reproduce it

Run the `sweep` subcommand over all nine Table 7-2 conditions (the 3×3 grid of $f, q \in \{5, 10, 15\}$), with 10 runs each:

```bash
cargo run --release -- sweep \
    --features-min 5 --features-max 15 --features-step 5 \
    --traits-min   5 --traits-max   15 --traits-step   5 \
    --runs 10 --seed 42
```

Then visualize the result. The visualization detects a `sweep` result automatically and, where applicable, draws a comparison against the Table 7-2 benchmark.

```bash
uv run python analysis/visualize.py
```

For the figures produced and how to interpret them, see [Visualization](visualization.md). For the full flag tables, see [CLI — `sweep`](cli.md#sweep-parameter-sweep).
