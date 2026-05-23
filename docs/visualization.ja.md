[English](visualization.md) | **日本語**

# 可視化 (Python)

Python ツールは単一の統一スクリプト `analysis/visualize.py` です．Python 依存管理には [uv](https://docs.astral.sh/uv/) を使用します．

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
├── simulate_distribution.png   # 安定地域数の分布（箱ひげ＋ジッタ）
├── simulate_metrics.png        # 各指標の平均±95%CI
└── simulate_vs_table7_2.png    # Table 7-2 のベンチマークとの比較（該当時のみ）
```

## sweep の可視化

結果が `sweep` の場合，同じコマンドで f×q スイープの図を生成します．

**出力ファイル（sweep）:**

```
results/latest/figures/
├── sweep_heatmap_regions.png   # 平均 n_stable_regions の f×q ヒートマップ
├── sweep_heatmap_ci.png        # 95%CI の f×q ヒートマップ
├── sweep_marginal_features.png # f を X 軸とする周辺折れ線（q ごと）
├── sweep_marginal_traits.png   # q を X 軸とする周辺折れ線（f ごと）
└── sweep_overview.png          # 2×2 概要パネル
```

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
