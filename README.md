# replication-dissemination-of-culture

Axelrod (1997) "The Dissemination of Culture: A Model with Local Convergence and Global Polarization" の再現実験実装．

## 参照論文

Axelrod, R. (1997). The Dissemination of Culture: A Model with Local Convergence and Global Polarization.
*Journal of Conflict Resolution*, 41(2), 203–226.

---

## モデル概要

- **状態**: `width × height` の格子．各サイトは長さ `f`（特徴数）の文化ベクトルを持ち，各要素は `0..q`（特性数）の整数．
- **1イベント**:
  1. アクティブサイト $s$ を一様ランダムに選ぶ．
  2. $s$ のフォン・ノイマン近傍（非トーラス，4近傍）から隣人 $nb$ を一様ランダムに選ぶ．
  3. 類似度 $\mathrm{sim} = (\text{一致特徴数}) / f$ を計算．
  4. 確率 $\mathrm{sim}$ で相互作用．差がある特徴を1つランダムに選び，$s$ が $nb$ の値をコピー．
  5. $\mathrm{sim} \in \{0, 1\}$ のときは相互作用しない．
- **安定条件**: 全ての隣接ペアについて $\mathrm{sim} \in \{0, 1\}$．
- **主要指標**: 安定文化地域数（同一文化ベクトルの4連結成分の数）．

---

## 実行方法

### 1. シミュレーション (Rust)

```bash
# ビルド
cargo build --release

# 統合テストの実行（similarity / is_stable / count_stable_regions /
# random_init / 小規模 e2e 収束を検証．tests/integration_test.rs）
cargo test --release

# 単一パラメータでの実行（Table 7-2 ベースケース, f=5, q=10, 10×10, 10回）
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
├── latest -> simulate_20260415_120000        ← 最新実行へのシンボリックリンク
├── simulate_20260415_120000/
│   ├── config.json                          ← 実行時設定（subcommand="simulate"）
│   └── metrics.csv                          ← 各 run の指標
└── ...
```

### 2. パラメータスイープ（感度分析）

特徴数 $f$ と特性数 $q$ のグリッドサーチを行う．

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
│   ├── config.json                          ← 実行時設定（subcommand="sweep"）
│   └── metrics.csv                          ← 全 (f, q, run) の結果
└── ...
```

simulate / sweep いずれもディレクトリ名が `simulate_<ts>` / `sweep_<ts>` と対称で，設定は `config.json`，結果は `metrics.csv` に統一されている．

### 3. 可視化 (Python)

Python 依存管理には [uv](https://docs.astral.sh/uv/) を使用．

```bash
# 依存パッケージのインストール
uv sync

# 結果の可視化（results/latest を自動参照）
# visualize.py は simulate / sweep 両対応の統一スクリプト．
# config.json の subcommand フィールド（無い場合はディレクトリ名の
# simulate_ / sweep_ 接頭辞）からモードを自動判定する．
uv run python analysis/visualize.py

# 特定の実行結果を可視化（simulate でも sweep でも同じコマンド）
uv run python analysis/visualize.py --results_dir results/simulate_20260415_120000
uv run python analysis/visualize.py --results_dir results/sweep_20260415_120500
```

**出力ファイル（simulate）:**

```
results/latest/figures/
├── simulate_distribution.png   ← 安定地域数の分布（箱ひげ＋ジッタ）
├── simulate_metrics.png        ← 各指標の平均±95%CI
└── simulate_vs_table7_2.png    ← Table 7-2 のベンチマークとの比較（該当時のみ）
```

**出力ファイル（sweep）:**

```
results/latest/figures/
├── sweep_heatmap_regions.png   ← 平均 n_stable_regions の f×q ヒートマップ
├── sweep_heatmap_ci.png        ← 95%CI の f×q ヒートマップ
├── sweep_marginal_features.png ← f を X 軸とする周辺折れ線（q ごと）
├── sweep_marginal_traits.png   ← q を X 軸とする周辺折れ線（f ごと）
└── sweep_overview.png          ← 2×2 概要パネル
```

---

## 論文のベンチマーク (Table 7-2)

10×10グリッド，各条件10回の平均安定文化地域数．

| 特徴数 $f$ | 特性数 $q$ | 論文値 | 再現目標 |
|-----------|-----------|-------|---------|
| 5 | 5 | 1.0 | ±0.3 |
| 5 | 10 | 3.2 | ±0.5 |
| 5 | 15 | 20.0 | ±3.0 |
| 10 | 5 | 1.0 | ±0.3 |
| 10 | 10 | 1.0 | ±0.3 |
| 10 | 15 | 1.4 | ±0.3 |
| 15 | 5 | 1.0 | ±0.3 |
| 15 | 10 | 1.0 | ±0.3 |
| 15 | 15 | 1.2 | ±0.3 |

この表から観察される2次元の非対称的効果:

- $f$（特徴数）の増加 → 地域数は **減少**（反直感的）．共通点が増えるほど類似度が高まり収斂しやすい．
- $q$（特性数）の増加 → 地域数は **増加**（直感的）．初期多様性が高まり共通点が見つかりにくい．

---

## 出力の解釈

### metrics.csv のカラム（simulate / sweep 共通）

| カラム | 説明 |
|-------|------|
| `run` | 試行番号 (0-indexed) |
| `width`, `height` | グリッドサイズ |
| `features`, `traits` | 特徴数 $f$・特性数 $q$ |
| `seed` | その試行で使用した派生シード |
| `converged` | 安定状態に到達したか（true/false） |
| `n_events` | 実際に実行したイベント数 |
| `n_stable_regions` | 安定文化地域数（4連結成分の数）．**主要指標** |
| `max_region_size` | 最大地域のサイズ（サイト数） |
| `n_distinct_cultures` | 盤面上に現れる相異なる文化ベクトルの数 |

### 典型的な結果の読み方

- **$f$ が大きく $q$ が小さい** → 地域は 1 に収斂し単一文化になる．
- **$f$ が小さく $q$ が大きい** ($f=5, q=15$) → 多数の安定地域が残存し，グローバルな文化的分極化が内生的に生じる．
- **収束判定**: `converged=true` なら全隣接ペアで $\mathrm{sim} \in \{0, 1\}$ が成立している．`false` の場合は `--max-events` を増やして再実行する．

---

## アルゴリズムの設計判断

- **乱数**: `ChaCha8Rng` を用いて決定的再現性を保証．
- **派生シード**: ベースシード $s$ から `s * 1_000_003 + f * 10_007 + q * 101 + run` として各試行のシードを決定的に派生させる．同じベースシードで `sweep` を繰り返すと同じ結果が得られる．
- **境界**: 非トーラス．端のサイトの近傍数は 2 または 3．論文 Table 7-2 は10×10の有限盤面で測定されているため境界効果を含む．
- **安定判定**: 毎イベントで走査するのは $O(n)$ のコストがかかるため，$n = \text{width} \times \text{height}$ イベントごとに全隣接ペアを走査して判定する．最悪ケースでは `--max-events` でカットオフされる．

---

## ライセンス

MIT

---
*This file was generated by Claude Code.*
