[English](visualization.md) | **日本語**

# 可視化 (Python)

Python ツールは単一の統一スクリプト `analysis/visualize.py` です．Python 依存管理には [uv](https://docs.astral.sh/uv/) を使用します．

```bash
# 依存パッケージのインストール
uv sync

# 直近の simulate を可視化
# run の選択は runvault に任せる:
#   runvault path --experiment axelrod --latest --subcommand simulate --standalone
# --standalone が既定．sweep の子は手で起こした run と subcommand が同じなので，
# これが無いと «最後に走った子» が返る．
# モードは run.json の subcommand から決まる（ディレクトリ名からは推定しない）．
uv run python analysis/visualize.py

# sweep の子も候補に入れる場合
uv run python analysis/visualize.py --include-sweep-children

# 直近の sweep（親 run）を可視化
# --subcommand を付けないと --latest が sweep 親を掴むことがあるので必ず指定する．
uv run python analysis/visualize.py --subcommand sweep

# results ルートが既定 (results) でない場合
uv run python analysis/visualize.py --results_root /tmp/rv-axelrod

# run ディレクトリを直接指定する（simulate でも sweep でも同じコマンド）
uv run python analysis/visualize.py --results_dir results/axelrod/simulate_20260415_120000_9f2c41ab_3b1d
```

`runvault` が PATH に無い場合は，環境変数 `RUNVAULT` で実行ファイルを指してください．

```bash
RUNVAULT=~/Documents/workspace/rust/rs-runvault/target/debug/runvault \
    uv run python analysis/visualize.py
```

**出力ファイル（simulate）:**

図は run ディレクトリの**外**に出ます．`manifest.csv` は `finish()` が確定させているので，後から作る図を run の中に置くと記録と食い違います．

```
results/axelrod/figures/<run_slug>/
├── simulate_distribution.png   # 安定地域数の分布（箱ひげ＋ジッタ）
├── simulate_metrics.png        # 各指標の平均±95%CI
└── simulate_vs_table7_2.png    # Table 7-2 のベンチマークとの比較（該当時のみ）
```

## sweep の可視化

結果が `sweep` の場合，同じコマンドで f×q スイープの図を生成します．

**出力ファイル（sweep）:**

```
results/axelrod/figures/<run_slug>/
├── sweep_heatmap_regions.png   # 平均 n_stable_regions の f×q ヒートマップ
├── sweep_heatmap_ci.png        # 95%CI の f×q ヒートマップ
├── sweep_marginal_features.png # f を X 軸とする周辺折れ線（q ごと）
├── sweep_marginal_traits.png   # q を X 軸とする周辺折れ線（f ごと）
└── sweep_overview.png          # 2×2 概要パネル
```

## 出力の解釈

### events.jsonl の 1 行（試行 1 本．`schema` は `terminal`）

| フィールド | 説明 |
|-------|------|
| `unit_id` | 観測主体．`trial-N`（N は 0 始まりの試行番号） |
| `t` | その試行が終わったイベント数．**予約語** |
| `t_unit` | `t` の単位．`event`（Axelrod のモデル時間の刻みは 1 イベント）．**予約語** |
| `outcome` | `converged` / `unconverged`．**予約語** |
| `censored` | 収束せず上限に達したか．真なら `t == budget`．**予約語** |
| `budget` | 上限イベント数（`--max-events`）．**予約語** |
| `seed` | その試行で使用した派生シード |
| `n_stable_regions` | 安定文化地域数（4連結成分の数）．**主要指標** |
| `max_region_size` | 最大地域のサイズ（サイト数） |
| `n_distinct_cultures` | 盤面上に現れる相異なる文化ベクトルの数 |

グリッドサイズ・$f$・$q$ は試行ごとに変わらないので，イベント行ではなく `config.json` の `parameters` にあります．

### metrics.csv の行（run 全体の集約．`scope=run`）

| 指標名 | 説明 |
|-------|------|
| `n_units` | 試行数．**runvault の予約指標名** |
| `n_converged` | 収束した試行数 |
| `convergence_rate` | 収束率 |
| `mean_n_stable_regions` | 安定文化地域数の平均．**主要指標** |
| `mean_max_region_size` | 最大地域サイズの平均 |
| `mean_n_distinct_cultures` | 相異なる文化数の平均 |
| `mean_n_events` | 実行イベント数の平均 |

### 典型的な結果の読み方

- **$f$ が大きく $q$ が小さい** → 地域は 1 に収斂し単一文化になる．
- **$f$ が小さく $q$ が大きい** ($f=5, q=15$) → 多数の安定地域が残存し，グローバルな文化的分極化が内生的に生じる．
- **収束判定**: `outcome=converged` なら全隣接ペアで $\mathrm{sim} \in \{0, 1\}$ が成立している．`unconverged`（＝ `censored=true`）の場合は `--max-events` を増やして再実行する．
