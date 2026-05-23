<p align="center"><img src="docs/assets/hero.svg" width="100%"></p>

**English** | [日本語](README.ja.md)

# The Dissemination of Culture — Axelrod (1997)

A reimplementation of the culture-dissemination model from Axelrod (1997), "The Dissemination of Culture: A Model with Local Convergence and Global Polarization." Sites on a 2D lattice carry culture vectors and imitate similar neighbors; mild local interaction produces local convergence into cultural regions while the global pattern can stay polarized. The simulation is written in Rust on the socsim framework and the visualization tools in Python.

## Install & Quick start

```bash
# Build the Rust simulation
cargo build --release

# Run the integration tests
cargo test --release

# Run a single parameter set (Table 7-2 base case: f=5, q=10, 10×10, 10 runs)
cargo run --release -- simulate --features 5 --traits 10 --runs 10 --seed 42

# Install the Python visualization tools
uv sync

# Visualize the most recent run (results/latest is auto-detected)
uv run python analysis/visualize.py
```

## Documentation

- [Use cases](docs/usecases.md) — what you can do with this project, with pointers to the rest of the docs.
- [CLI](docs/cli.md) — the Rust CLI: the `simulate` and `sweep` subcommands and the `results/` output layout.
- [Reproduction](docs/reproduction.md) — the Table 7-2 benchmark and the f×q asymmetry, and how to reproduce it.
- [Visualization](docs/visualization.md) — the Python `analysis/visualize.py` and how to interpret the outputs.
- [Architecture](docs/architecture.md) — repository structure, the socsim framework, references, and design decisions.

## License

MIT
