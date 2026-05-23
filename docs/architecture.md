**English** | [日本語](architecture.ja.md)

# Architecture

## Repository structure

A two-project layout: a Cargo workspace + a uv project.

```
axelrod1997/
├── Cargo.toml                 # Cargo workspace root
├── pyproject.toml             # uv project root
├── simulation/                # Rust project (package axelrod-culture, bin axelrod)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                    # CLI (simulate / sweep)
│   │   ├── lib.rs                     # library crate root
│   │   ├── config.rs                  # configuration types
│   │   ├── world.rs                   # socsim WorldState impl (CellGrid<Culture> + precomputed Adjacency)
│   │   ├── mechanisms.rs              # socsim Mechanism impl (the Axelrod event rule + stability check)
│   │   ├── metrics.rs                 # stable-region count, max region size, distinct cultures
│   │   └── simulation.rs              # run driver (Simulation::run_observed, early stop on convergence)
│   └── tests/
│       └── integration_test.rs        # integration tests (run via `cargo test`)
├── analysis/                  # Python project
│   └── visualize.py                   # unified simulate / sweep visualization
└── results/                   # simulation output (gitignored)
```

- `cargo run --release -- <subcommand>` launches the `axelrod` binary of the `axelrod-culture` crate from the workspace root.
- `uv run python analysis/visualize.py` runs the unified visualization script.

## The model on the socsim framework

The simulation is built on top of the social-simulation framework [rs-social-simulation-tools](https://github.com/akitenkrad/rs-social-simulation-tools) (socsim) — a git dependency on the `socsim-core`, `socsim-engine`, and `socsim-grid` crates, with the commit pinned in `Cargo.lock`.

### World

The world (`AxelrodWorld`) implements socsim's `WorldState`. Site state lives in a `socsim_grid::CellGrid<Culture>` — one per-cell culture vector per site (flat index `idx = r*cols + c`), the single source of truth for the board. Because every cell is occupied and no agent ever moves (only the culture vectors mutate), the occupancy index is not used; the grid is used only for neighbor/topology computation.

The grid is constructed with `Boundary::Fixed` (non-toroidal) and the von Neumann neighborhood (`Neighborhood::VonNeumann`, 4-neighbor). A CSR neighbor table (`socsim_grid::Adjacency`) is precomputed once via `Grid::adjacency(...)` and reused in the hot loop, so neighbor lookups are cheap.

### Mechanism and driver

The model logic is a socsim `Mechanism` (`AxelrodInteractionMechanism`) that fires in the `Phase::Interaction` phase. On each engine step it batches `n_sites` (= `width × height`) micro-events; one micro-event picks an active site, picks a neighbor, computes similarity, and with probability $\mathrm{sim}$ copies a differing feature. After the batch it checks global stability ($\mathrm{sim} \in \{0, 1\}$ for every adjacent pair) and, on convergence, calls `StepContext::request_stop`. Per-step results (events executed, convergence flag) are passed to the driver via `StepContext::scratch`.

The driver (`run`) builds the engine with `SimulationBuilder` and uses `Simulation::run_observed(...)` to step the engine, collect per-step metrics from `scratch`, and stop on the convergence request. With `events_per_step = n_sites`, `t_max = ceil(max_events / events_per_step)`.

### Determinism and seed derivation

- **RNG**: `socsim_core::SimRng` (a ChaCha20-based generator) guarantees deterministic, reproducible runs.
- **Seed derivation**: per-run seeds are derived deterministically with `socsim_core::derive_seed(base, &[features, traits, run])`. Repeating a `sweep` with the same base seed yields identical results.
- **RNG stream separation**: from a single root seed, independent labeled streams are derived — `derive_seed(root, &[0])` for world initialization (the random assignment of culture vectors) and `derive_seed(root, &[1])` for the engine (the event RNG inside the mechanism). Keeping the streams separate decouples the initial board from the dynamics.

## Design decisions

- **Boundary**: non-toroidal (`Boundary::Fixed`). Edge sites have 2 or 3 neighbors. Table 7-2 in the paper is measured on a finite 10×10 board, so boundary effects are intentionally included.
- **Stability-check cadence**: scanning every adjacent pair is $O(n)$, so the check is performed once per engine step (i.e. once every `n_sites` events) rather than after every micro-event. In the worst case a run is cut off by `--max-events`.
- **Determinism over bit-exactness**: reproducibility for a given seed (determinism) and qualitative reproduction of the paper are guaranteed; the consumed random-number sequence is an implementation detail of the socsim engine.

## References

Axelrod, R. (1997). The Dissemination of Culture: A Model with Local Convergence and Global Polarization.
*Journal of Conflict Resolution*, 41(2), 203–226.
DOI: [10.1177/0022002797041002001](https://doi.org/10.1177/0022002797041002001)
