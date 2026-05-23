[English](cli.md) | **日本語**

# Rust CLI

`axelrod-culture` クレート（バイナリ `axelrod`）は，`simulate` と `sweep` の2つのサブコマンドを持つ CLI を提供します．`cargo build --release` で一度ビルドし，workspace ルートから `cargo run --release -- <サブコマンド> ...` で実行します．

## モデル概要

`width × height` の格子の各サイトは長さ `f`（特徴数）の文化ベクトルを持ち，各要素は `0..q`（特性数）の整数です．1イベントでは，アクティブサイト $s$ を一様ランダムに選び，$s$ のフォン・ノイマン近傍（非トーラス，4近傍）から隣人 $nb$ を一様ランダムに選び，類似度 $\mathrm{sim} = (\text{一致特徴数}) / f$ を計算し，確率 $\mathrm{sim}$ で相互作用します — 差がある特徴を1つランダムに選び，$s$ が $nb$ の値をコピーします．$\mathrm{sim} \in \{0, 1\}$ のときは相互作用しません．盤面は全ての隣接ペアで $\mathrm{sim} \in \{0, 1\}$ のとき安定し，主要指標は安定文化地域数（同一文化ベクトルの4連結成分の数）です．

## `simulate`（単一パラメータセット）

単一の特徴数・特性数の設定を，指定回数だけ実行します．

```bash
# ビルド
cargo build --release

# 統合テストの実行（similarity / is_stable / count_stable_regions /
# random_init / 小規模 e2e 収束を検証．tests/integration_test.rs）
cargo test --release

# 単一パラメータでの実行（Table 7-2 ベースケース: f=5, q=10, 10×10, 10回）
cargo run --release -- simulate \
    --features 5 --traits 10 \
    --runs 10 --seed 42
```

**`simulate` サブコマンドのオプション:**

| オプション | デフォルト | 説明 |
|-----------|-----------|------|
| `--width` | 10 | グリッド幅 |
| `--height` | 10 | グリッド高さ |
| `--features` / `-f` | 5 | 特徴数 $f$ |
| `--traits` / `-q` | 10 | 特性数 $q$ |
| `--runs` | 10 | 試行回数 |
| `--max-events` | 1000000 | 1試行あたりの最大イベント数 |
| `--seed` | — | 乱数シード（ベース値．省略時はランダム） |
| `--output-dir` | `results` | 出力先ディレクトリ |

**出力ファイル:**

```
results/
├── latest -> simulate_20260415_120000        # 最新実行へのシンボリックリンク
├── simulate_20260415_120000/
│   ├── config.json                           # 実行時設定（subcommand="simulate"）
│   └── metrics.csv                           # 各 run の指標
└── ...
```

## `sweep`（パラメータスイープ）

特徴数 $f$ と特性数 $q$ のグリッドサーチを行います．

```bash
# Table 7-2 全条件 (3×3) を 10 runs ずつで再現
cargo run --release -- sweep \
    --features-min 5 --features-max 15 --features-step 5 \
    --traits-min   5 --traits-max   15 --traits-step   5 \
    --runs 10 --seed 42
```

**`sweep` サブコマンドのオプション:**

| オプション | デフォルト | 説明 |
|-----------|-----------|------|
| `--width` | 10 | グリッド幅 |
| `--height` | 10 | グリッド高さ |
| `--features-min` | 5 | 特徴数 $f$ の開始値 |
| `--features-max` | 15 | 特徴数 $f$ の終了値（含む） |
| `--features-step` | 5 | 特徴数 $f$ の刻み幅 |
| `--traits-min` | 5 | 特性数 $q$ の開始値 |
| `--traits-max` | 15 | 特性数 $q$ の終了値（含む） |
| `--traits-step` | 5 | 特性数 $q$ の刻み幅 |
| `--runs` | 10 | 各条件あたりの試行回数 |
| `--max-events` | 1000000 | 1試行あたりの最大イベント数 |
| `--seed` | — | 乱数シード（ベース値） |
| `--output-dir` | `results` | 出力先ベースディレクトリ |

**出力ファイル:**

```
results/
├── latest -> sweep_20260415_120500
├── sweep_20260415_120500/
│   ├── config.json                           # 実行時設定（subcommand="sweep"）
│   └── metrics.csv                           # 全 (f, q, run) の結果
└── ...
```

`simulate` / `sweep` いずれもディレクトリ名が `simulate_<ts>` / `sweep_<ts>` と対称で，設定は `config.json`，結果は `metrics.csv` に統一されています．

`metrics.csv` のカラム参照と図の読み方については [可視化](visualization.ja.md) を参照してください．
