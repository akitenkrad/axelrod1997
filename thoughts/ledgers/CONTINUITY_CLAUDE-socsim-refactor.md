# Continuity Ledger — axelrod1997 socsim リファクタ

## Goal

axelrod1997（Axelrod 1997 文化拡散モデルの再現実装）を、ローカルライブラリ
`rs-social-simulation-tools`（socsim）の上に載せ替える．schelling1971 と同一の統合
パターンに揃える．

**Done の定義:**

- `socsim-core` / `socsim-engine` / `socsim-grid` の3クレートに依存し，
  `WorldState` + `Mechanism` + `SimulationBuilder` でモデルが動く．
- `cargo build --release` が通り，`simulate` / `sweep` の CLI・CSV 列・`config.json`・
  `latest` symlink が**従来どおり**（Python 可視化を壊さない）．
- Table 7-2 代表条件が許容範囲で再現（f=5,q=10→約3.2 / f=5,q=15→約20 / f=15,q=5→約1）．

## Constraints

- 手本は schelling1971（同じグリッド系・同じライブラリ）．依存宣言・レイアウト・
  「CLI は独自 clap を維持し `run(cfg) -> RunResult` を継ぎ目にする」方針を踏襲．
- 使う socsim クレートは **3つのみ**（core / engine / grid）．socsim-cli / -config /
  -runner / -log は使わない（schelling と同じ）．
- CSV スキーマ（`SimulateRow` / `SweepRow` の列）と出力レイアウトは不変に保つ．
- `analysis/*.py` と `pyproject.toml` / `uv.lock` は無改修．

## Key Decisions

- **依存方法 = git 依存**（schelling と同一）．
  `socsim-core = { git = "https://github.com/akitenkrad/rs-social-simulation-tools", branch = "main", package = "socsim-core" }`
  同様に `-engine` / `-grid`．commit は Cargo.lock で固定．
- **イベント駆動の載せ方 = n_sites イベント/ステップのバッチ．**
  1 エンジン step = `events_per_step`（既定 = `n_sites` = 旧 `check_interval`）回のイベント．
  `t_max = ceil(max_events / n_sites)`．元コードも check_interval 境界でしか安定判定
  しないため**論理的に等価**かつエンジン overhead 最小．スケジューラの活性化順序は
  使わない（イベントは内部でランダムにサイト選択）ので最安の `SequentialScheduler`．
- **RNG の帰結:** `ChaCha8Rng` → `SimRng`(ChaCha20)．旧実装とのビット一致は失われるが
  決定的再現性は維持．論文 Table 7-2 の許容誤差再現が目標（schelling と同じ割り切り）．
- 手書き `derive_seed` → `socsim_core::derive_seed(base, &[features, traits, run])` に置換．
- 近傍は init 時に `Grid::neighbors(r,c,VonNeumann)`（`Boundary::Fixed`）で事前計算し
  `Vec<Vec<usize>>`（フラット index）で保持 → ホットループ高速化．
- `socsim-grid::GridIndex` は使わない（Axelrod はエージェント移動なし・全セル占有・
  文化ベクトルが mutate するだけなので占有インデックスは不適）．`Grid` は近傍/位相にのみ使用．

## State

- Done:
  - [x] 既存 axelrod1997 実装の把握（grid/culture/metrics/config/main）
  - [x] socsim ライブラリ API 面の把握（core/engine/grid/rng/runner/config/log/cli）
  - [x] schelling1971 統合パターンの把握（手本）
  - [x] リファクタ計画の立案・主要設計判断の確定（依存=git / バッチ=n_sites）
  - [x] Phase 0: workspace 化（root Cargo.toml members=["simulation"]，src/→simulation/src/，git 依存追加，rand_chacha 削除）
  - [x] Phase 1: world.rs（旧 grid.rs 吸収．AxelrodWorld + WorldState impl + 近傍事前計算 + random_init）
  - [x] Phase 2: mechanisms.rs（similarity/is_stable 移植，AxelrodInteractionMechanism, Phase::Interaction, scratch + request_stop）
  - [x] Phase 3: simulation.rs（run(...)->RunResult，SimulationBuilder + 手動 step ループ + scratch 集計）
  - [x] Phase 4: metrics.rs（count_stable_regions を world+事前計算近傍に微修正）
  - [x] Phase 5: main.rs/config.rs（derive_seed→socsim_core::derive_seed，execute_run→simulation::run，CLI/CSV/json/symlink 維持）
  - [x] Phase 6: 検証（cargo build --release クリーン / simulate 3条件で f×q 非対称を確認 / sweep 動作確認 / CSV・config.json・latest symlink スキーマ不変）
- Now: [→] リファクタ完了．未コミット（指示によりコミットしない）．
- Next: （任意）README/docs の bilingual 化判断，Python 可視化での目視確認
- Remaining:
  - [ ] （任意）README/docs を schelling 流の bilingual + docs/ 構成に揃える
  - [ ] （任意）`uv run python analysis/visualize.py` での可視化目視確認

## Open Questions

- RESOLVED: github の rs-social-simulation-tools の main に core/engine/grid の現行 API は
  push 済み．`cargo build` が git 依存(commit 0aac58c)を解決し `cargo build --release` が
  クリーンに通った（core/engine/grid/rng/log を取得・コンパイル成功）．
- UNCONFIRMED: README/docs を schelling 流の bilingual + docs/ 構成に揃えるかは未着手（任意）．

## Working Set

- 対象: `/Users/akitenkrad/Documents/workspace/simulations/axelrod1997`
- 手本: `/Users/akitenkrad/Documents/workspace/simulations/schelling1971`（特に simulation/src/{world,mechanisms,simulation,main}.rs と thoughts/ledgers/CONTINUITY_CLAUDE-socsim-refactor.md）
- ライブラリ: `/Users/akitenkrad/Documents/workspace/simulations/rs-social-simulation-tools`
- 旧→新ファイル対応: grid.rs→world.rs / culture.rs→mechanisms.rs+simulation.rs / metrics.rs→metrics.rs / config.rs→config.rs / main.rs→main.rs
- ビルド/実行: `cargo build --release` ; `cargo run --release -- simulate -f 5 -q 10 --runs 10 --seed 42`
- 可視化: `uv run python analysis/visualize.py`

## Post-refactor: stash 取り込み

リファクタ後の canonical リポジトリへ，pre-refactor WIP（git stash）の選別取り込みを実施．

- **出力命名の統一**: simulate 出力ディレクトリを `<ts>` → `simulate_<ts>`，sweep を `<ts>_sweep` → `sweep_<ts>` と対称化．sweep の設定ファイルを `sweep_config.json` → `config.json`，サマリを `sweep_summary.csv` → `metrics.csv` に統一（simulate と完全対称）．CSV カラムスキーマは不変．`config.json` の `subcommand` フィールド（simulate/sweep）と `results/latest` シンボリックリンク挙動は維持．
- **可視化の統一**: `analysis/visualize.py` を stash 版（simulate / sweep 両対応・`config.json` の `subcommand` か `simulate_`/`sweep_` 接頭辞で自動判定）に置換．旧 `analysis/visualize_sweep.py` を削除．`seaborn` は pyproject.toml に既存（追加不要）．simulate / sweep 両ディレクトリで PNG 生成を実機確認済み．
- **lib ターゲット追加**: `simulation/src/lib.rs`（`config`/`world`/`mechanisms`/`simulation`/`metrics` を公開）を新設し，Cargo.toml に `[lib] name="axelrod_culture"` を追加．`main.rs` は `mod` 宣言を `use axelrod_culture::{config, metrics, simulation};` に置換（main が実際に使うモジュールのみ）．
- **統合テスト追加**: `simulation/tests/integration_test.rs` を新設．stash 版 10 テストを新 API へ移植（`culture::*`→`mechanisms::*`，`Grid`→`AxelrodWorld`（手構築は `AxelrodWorld::new` を使用），`ChaCha8Rng`→`SimRng::from_seed`，e2e は `simulation::run`）．`cargo test --release` で 10/10 パス．

新規ヘルパ追加なし（既存の公開 `AxelrodWorld::new` で手構築テストを実現）．`cargo build --release` クリーン（警告なし）．

---
*This file was generated by Claude Code.*
