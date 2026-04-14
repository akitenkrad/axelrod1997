#!/usr/bin/env python3
"""
visualize.py — Axelrod (1997) 文化拡散モデル `simulate` 結果 可視化スクリプト

Usage:
    uv run python analysis/visualize.py
    uv run python analysis/visualize.py --results_dir results/20260405_153000
    uv run python analysis/visualize.py --results_dir results/latest --output_dir out

Outputs:
    output_dir/
    ├── simulate_distribution.png   ← n_stable_regions の分布（箱ひげ＋ジッタ）
    ├── simulate_metrics.png        ← 各指標のサマリ（平均±95%CI）
    └── simulate_vs_table7_2.png    ← Table 7-2 のベンチマークとの比較（該当する場合のみ）
"""

from __future__ import annotations

import argparse
import json
import os
import sys

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# --------------------------------------------------------------------------- #
# 日本語フォント設定
# --------------------------------------------------------------------------- #
plt.rcParams["font.family"] = "Hiragino Sans"

COLOR_BG = "#FAFAF8"
COLOR_MAIN = "#4c97c9"
COLOR_ACCENT = "#F44336"
COLOR_REF = "#9C27B0"


# Table 7-2 (Axelrod 1997): 10×10 グリッド，10 回平均の安定地域数
TABLE_7_2 = {
    (5, 5): 1.0,
    (5, 10): 3.2,
    (5, 15): 20.0,
    (10, 5): 1.0,
    (10, 10): 1.0,
    (10, 15): 1.4,
    (15, 5): 1.0,
    (15, 10): 1.0,
    (15, 15): 1.2,
}


# --------------------------------------------------------------------------- #
# ユーティリティ
# --------------------------------------------------------------------------- #


def load_config(results_dir: str) -> dict | None:
    path = os.path.join(results_dir, "config.json")
    if os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    return None


def summarize(df: pd.DataFrame) -> dict:
    """各指標の平均と95%CIを計算する"""
    out = {}
    for col in ("n_stable_regions", "max_region_size", "n_distinct_cultures", "n_events"):
        if col not in df.columns:
            continue
        vals = df[col].astype(float).values
        n = len(vals)
        mean = float(np.mean(vals))
        std = float(np.std(vals, ddof=1)) if n > 1 else 0.0
        ci95 = 1.96 * std / np.sqrt(n) if n > 1 else 0.0
        out[col] = {"mean": mean, "std": std, "ci95": ci95, "n": n}
    return out


# --------------------------------------------------------------------------- #
# 描画
# --------------------------------------------------------------------------- #


def plot_distribution(df: pd.DataFrame, out_path: str, subtitle: str) -> None:
    fig, ax = plt.subplots(figsize=(8, 5), facecolor=COLOR_BG)
    ax.set_facecolor(COLOR_BG)

    vals = df["n_stable_regions"].dropna().values

    ax.boxplot(
        vals,
        vert=True,
        patch_artist=True,
        boxprops=dict(facecolor=COLOR_MAIN, alpha=0.6),
        medianprops=dict(color=COLOR_ACCENT, linewidth=2),
        widths=0.5,
    )
    # ジッタ付き個別点
    xs = np.random.normal(loc=1.0, scale=0.04, size=len(vals))
    ax.scatter(xs, vals, color=COLOR_MAIN, alpha=0.7, s=30, zorder=3)

    ax.set_xticks([1])
    ax.set_xticklabels(["n_stable_regions"])
    ax.set_ylabel("安定文化地域数")
    ax.set_title("安定文化地域数の分布")
    ax.grid(True, alpha=0.3, axis="y")
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")

    fig.tight_layout(rect=[0, 0, 1, 0.92])
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out_path}")


def plot_metrics(df: pd.DataFrame, summary: dict, out_path: str, subtitle: str) -> None:
    """複数指標の平均と95%CIをバー形式で表示"""
    cols = [
        ("n_stable_regions", "安定地域数", COLOR_MAIN),
        ("max_region_size", "最大地域サイズ", "#4CAF50"),
        ("n_distinct_cultures", "相異なる文化数", COLOR_REF),
    ]
    fig, axes = plt.subplots(1, 3, figsize=(13, 4.5), facecolor=COLOR_BG)
    for ax, (col, label, color) in zip(axes, cols):
        ax.set_facecolor(COLOR_BG)
        if col not in summary:
            ax.set_visible(False)
            continue
        s = summary[col]
        ax.bar([0], [s["mean"]], yerr=[s["ci95"]],
               color=color, alpha=0.7, capsize=6, width=0.5)
        ax.set_xticks([0])
        ax.set_xticklabels([label])
        ax.set_title(f"{label}\n平均={s['mean']:.2f} ± {s['ci95']:.2f} (95%CI, n={s['n']})")
        ax.grid(True, alpha=0.3, axis="y")

    fig.suptitle("Axelrod 文化拡散モデル — simulate 指標サマリ", fontsize=13)
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")
    fig.tight_layout(rect=[0, 0, 1, 0.90])
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out_path}")


def plot_vs_table_7_2(df: pd.DataFrame, config: dict | None, out_path: str) -> bool:
    """config の (features, traits) が Table 7-2 に含まれる場合に比較プロットを保存"""
    if not config:
        return False
    f = config.get("features")
    q = config.get("traits")
    if f is None or q is None:
        return False
    key = (int(f), int(q))
    if key not in TABLE_7_2:
        return False

    paper = TABLE_7_2[key]
    vals = df["n_stable_regions"].astype(float).values
    if len(vals) == 0:
        return False
    mean = float(np.mean(vals))
    std = float(np.std(vals, ddof=1)) if len(vals) > 1 else 0.0
    ci95 = 1.96 * std / np.sqrt(len(vals)) if len(vals) > 1 else 0.0

    fig, ax = plt.subplots(figsize=(7, 5), facecolor=COLOR_BG)
    ax.set_facecolor(COLOR_BG)
    labels = ["Axelrod (1997)\nTable 7-2", f"本実装\n(n={len(vals)})"]
    means = [paper, mean]
    errs = [0.0, ci95]
    colors = [COLOR_REF, COLOR_MAIN]
    ax.bar(labels, means, yerr=errs, color=colors, alpha=0.75, capsize=6, width=0.5)
    ax.set_ylabel("安定文化地域数")
    ax.set_title(f"Table 7-2 比較 (f={f}, q={q}, 10×10)")
    ax.grid(True, alpha=0.3, axis="y")
    fig.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out_path}")
    return True


# --------------------------------------------------------------------------- #
# メイン
# --------------------------------------------------------------------------- #


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Axelrod 文化拡散モデル simulate 結果 可視化スクリプト"
    )
    p.add_argument(
        "--results_dir", default="results/latest",
        help="simulate 結果ディレクトリ (default: results/latest)",
    )
    p.add_argument(
        "--output_dir", default=None,
        help="図の保存先ディレクトリ (default: {results_dir}/figures)",
    )
    return p.parse_args()


def main() -> None:
    args = parse_args()
    results_dir = args.results_dir
    out_dir = args.output_dir if args.output_dir else os.path.join(results_dir, "figures")

    metrics_path = os.path.join(results_dir, "metrics.csv")
    if not os.path.exists(metrics_path):
        print(f"エラー: metrics.csv が見つかりません: {metrics_path}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(out_dir, exist_ok=True)

    print("=== Axelrod 文化拡散モデル 可視化 (simulate) ===")
    print(f"結果ディレクトリ: {results_dir}")
    print(f"出力先:           {out_dir}")
    print("-----------------------------------------------")

    df = pd.read_csv(metrics_path)
    config = load_config(results_dir)

    # サブタイトル作成
    subtitle_parts = []
    if config:
        w = config.get("width", "?")
        h = config.get("height", "?")
        f = config.get("features", "?")
        q = config.get("traits", "?")
        subtitle_parts.append(f"{w}×{h} グリッド")
        subtitle_parts.append(f"f={f}, q={q}")
        subtitle_parts.append(f"{len(df)} runs")
    subtitle = "，".join(subtitle_parts)

    summary = summarize(df)

    print(f"[1/3] 分布プロットを保存中 ...")
    plot_distribution(df, os.path.join(out_dir, "simulate_distribution.png"), subtitle)

    print(f"[2/3] 指標サマリを保存中 ...")
    plot_metrics(df, summary, os.path.join(out_dir, "simulate_metrics.png"), subtitle)

    print(f"[3/3] Table 7-2 との比較を保存中 ...")
    saved = plot_vs_table_7_2(
        df, config, os.path.join(out_dir, "simulate_vs_table7_2.png")
    )
    if not saved:
        print("  (f, q) が Table 7-2 ベンチマーク外のためスキップ")

    # サマリ表示
    print("-----------------------------------------------")
    print("指標サマリ:")
    for col, s in summary.items():
        print(f"  {col:<22} mean={s['mean']:.3f}  95%CI=±{s['ci95']:.3f}  n={s['n']}")
    print("-----------------------------------------------")
    print("完了．出力ファイル一覧:")
    for f in sorted(os.listdir(out_dir)):
        fpath = os.path.join(out_dir, f)
        if os.path.isfile(fpath):
            size_kb = os.path.getsize(fpath) / 1024
            print(f"  {f:40s} ({size_kb:6.1f} KB)")


if __name__ == "__main__":
    main()
