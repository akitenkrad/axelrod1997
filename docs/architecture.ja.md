[English](architecture.md) | **日本語**

# アーキテクチャ

## リポジトリ構成

Cargo workspace と uv プロジェクトの2プロジェクト構成です．

```
axelrod1997/
├── Cargo.toml                 # Cargo workspace ルート
├── pyproject.toml             # uv プロジェクトルート
├── simulation/                # Rust プロジェクト（パッケージ axelrod-culture, bin axelrod）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                    # CLI（simulate / sweep）
│   │   ├── lib.rs                     # ライブラリクレートのルート
│   │   ├── config.rs                  # 設定型
│   │   ├── world.rs                   # socsim WorldState 実装（CellGrid<Culture> + 事前計算 Adjacency）
│   │   ├── mechanisms.rs              # socsim Mechanism 実装（Axelrod イベント規則 + 安定判定）
│   │   ├── metrics.rs                 # 安定地域数・最大地域サイズ・相異なる文化数
│   │   └── simulation.rs              # 実行ドライバ（Simulation::run_observed, 収束時の早期停止）
│   └── tests/
│       └── integration_test.rs        # 統合テスト（`cargo test` で実行）
├── analysis/                  # Python プロジェクト
│   └── visualize.py                   # simulate / sweep 統一可視化
└── results/                   # シミュレーション出力（gitignore 対象）
```

- `cargo run --release -- <サブコマンド>` は workspace ルートから `axelrod-culture` クレートの `axelrod` バイナリを起動します．
- `uv run python analysis/visualize.py` は統一可視化スクリプトを実行します．

## socsim フレームワーク上のモデル

シミュレーションは社会シミュレーションフレームワーク [rs-social-simulation-tools](https://github.com/akitenkrad/rs-social-simulation-tools)（socsim）の上に構築されています — `socsim-core`，`socsim-engine`，`socsim-grid` クレートへの git 依存で，commit は `Cargo.lock` で固定されます．

### 世界状態

世界状態（`AxelrodWorld`）は socsim の `WorldState` を実装します．セル状態は `socsim_grid::CellGrid<Culture>` に保持され，各サイトに1つの文化ベクトルが対応します（フラットインデックス `idx = r*cols + c`）．これが盤面の単一の真実源です．全セルが占有され，エージェントは移動せず文化ベクトルが mutate するだけなので，占有インデックスは使わず，グリッドは近傍・位相の計算にのみ用います．

グリッドは `Boundary::Fixed`（非トーラス）とフォン・ノイマン近傍（`Neighborhood::VonNeumann`，4近傍）で構築します．CSR の近傍表（`socsim_grid::Adjacency`）を `Grid::adjacency(...)` で一度だけ事前計算してホットループで再利用するため，近傍参照が安価です．

### メカニズムとドライバ

モデルロジックは `Phase::Interaction` フェーズで発火する socsim `Mechanism`（`AxelrodInteractionMechanism`）です．1 エンジン step ごとに `n_sites`（= `width × height`）回のマイクロイベントをバッチ実行します — 1マイクロイベントはアクティブサイトを選び，隣人を選び，類似度を計算し，確率 $\mathrm{sim}$ で差がある特徴をコピーします．バッチ後にグローバル安定（全隣接ペアで $\mathrm{sim} \in \{0, 1\}$）を判定し，収束時に `StepContext::request_stop` を呼びます．ステップ結果（実行イベント数・収束フラグ）は `StepContext::scratch` でドライバに渡します．

ドライバ（`run`）は `SimulationBuilder` でエンジンを構築し，`Simulation::run_observed(...)` でエンジンを回して `scratch` からステップ毎の指標を収集し，収束要求でループを抜けます．`events_per_step = n_sites`，`t_max = ceil(max_events / events_per_step)` です．

### 決定性とシード派生

- **乱数**: `socsim_core::SimRng`（ChaCha20 ベースの生成器）が決定的で再現可能な実行を保証します．
- **シード派生**: 各試行のシードは `socsim_core::derive_seed(base, &[features, traits, run])` で決定的に派生されます．同じベースシードで `sweep` を繰り返すと同じ結果が得られます．
- **RNG ストリーム分離**: 単一の root シードから用途別の独立なラベル付きストリームを派生させます — `derive_seed(root, &[0])` を世界初期化（文化ベクトルのランダム割り当て）用，`derive_seed(root, &[1])` をエンジン（メカニズム内のイベント RNG）用とします．ストリームを分離することで初期盤面と動学を切り離します．

## 設計判断

- **境界**: 非トーラス（`Boundary::Fixed`）．端のサイトの近傍数は 2 または 3．論文 Table 7-2 は10×10の有限盤面で測定されているため，境界効果を意図的に含みます．
- **安定判定の頻度**: 全隣接ペアの走査は $O(n)$ なので，毎マイクロイベントではなく1 エンジン step ごと（= `n_sites` イベントごと）に判定します．最悪ケースでは `--max-events` でカットオフされます．
- **ビット完全性より決定性**: あるシードに対する再現性（決定性）と論文の定性的再現は保証されます．消費される乱数列は socsim エンジンの実装詳細です．

## 参照論文

Axelrod, R. (1997). The Dissemination of Culture: A Model with Local Convergence and Global Polarization.
*Journal of Conflict Resolution*, 41(2), 203–226.
DOI: [10.1177/0022002797041002001](https://doi.org/10.1177/0022002797041002001)
