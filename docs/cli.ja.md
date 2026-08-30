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
| `--output-dir` | `results` | results ルート（この下に `<experiment>/<run_slug>/` ができる） |

`--seed` を省略した場合もシードは実行時に実体化され，`config.json` の `parameters.seed` と `run.json` の `rng.master_seed` の両方に残ります．「記録されない乱数で走る」ことはありません．

**出力ファイル:**

```
results/                                          # --output-dir で指定する results ルート
└── axelrod/                                      # experiment
    ├── latest_finished
    ├── simulate_20260415_120000_9f2c41ab_3b1d/   # run（<subcommand>_<時刻>_<cfg8>_<exec4>）
    │   ├── run.json                              # run のメタデータ（lineage / rng / research）
    │   ├── config.json                           # 実験条件は ["parameters"] の下
    │   ├── metrics.csv                           # run 全体の集約指標（long 形式）
    │   ├── events.jsonl                          # 試行ごとの terminal 行
    │   ├── status.json / manifest.csv
    │   └── lock/                                 # Cargo.lock / uv.lock の写し
    └── figures/<run_slug>/                       # 図（run の外．作図は run の記録ではない）
```

run ディレクトリの名前は runvault が決めます．`--output-dir` に渡すのは run そのものではなく results ルートです．直近の完了 run は次で引けます．

```bash
runvault path --experiment axelrod --latest --subcommand simulate
```

試行（`--runs`）は子 run にはしません．1 つの run の中の観測主体として `events.jsonl` に 1 行ずつ記録されます（`unit_id = trial-N`，`t` = そのイベント数で `t_unit = event`，`budget` = `--max-events`，収束しなければ `censored = true`）．`censored` なら `t == budget` であることは runvault が書き込み時に検査するので，矛盾した行はファイルに届きません．run 全体の集約（収束率・地域数の平均など）は `metrics.csv` の `scope=run` 行です．

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
| `--output-dir` | `results` | results ルート（この下に `<experiment>/<run_slug>/` ができる） |

**出力ファイル:**

`sweep` は親 run 1 つと，条件 (f, q) ごとの子 run を作ります．親は `config.json` にグリッド定義だけを持ち，指標は書きません（`rng.master_seed` は null）．子は `subcommand = "simulate"` として，単体の `simulate` と同じ形で記録されます（`lineage.parent_run_uid` と `lineage.sweep_id` で親を指す）．子は experiment ディレクトリの下に親と並んで置かれます．

```
results/axelrod/
├── sweep_20260415_120500_6d945b66_86c4/      # 親（グリッド定義）
├── simulate_20260415_120500_17ccc5bc_7076/   # 子 (f=5, q=5)
├── simulate_20260415_120501_8c542dd1_48f3/   # 子 (f=5, q=10)
└── ...
```

各条件の結果は子 run の `events.jsonl` / `metrics.csv` にあります．図の読み方については [可視化](visualization.ja.md) を参照してください．
