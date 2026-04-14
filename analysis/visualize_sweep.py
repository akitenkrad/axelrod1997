#!/usr/bin/env python3
"""
visualize_sweep.py — Axelrod (1997) 文化拡散モデル `sweep` 結果 可視化スクリプト

Usage:
    uv run python analysis/visualize_sweep.py
    uv run python analysis/visualize_sweep.py --sweep_dir results/20260405_160827_sweep
    uv run python analysis/visualize_sweep.py --sweep_dir results/latest --output_dir out

Outputs:
    output_dir/
    ├── sweep_heatmap_regions.png   ← 平均 n_stable_regions の f×q ヒートマップ
    ├── sweep_heatmap_ci.png        ← 95%CI の f×q ヒートマップ
    ├── sweep_marginal_features.png ← f を X 軸とする周辺折れ線（q ごと）
    ├── sweep_marginal_traits.png   ← q を X 軸とする周辺折れ線（f ごと）
    └── sweep_overview.png          ← 2×2 概要パネル
"""

from __future__ import annotations

import argparse
import json
import os
import sys

import matplotlib.cm as cm
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns

# --------------------------------------------------------------------------- #
# 日本語フォント設定
# --------------------------------------------------------------------------- #
plt.rcParams["font.family"] = "Hiragino Sans"

COLOR_BG = "#FAFAF8"
METRIC_COL = "n_stable_regions"

# --------------------------------------------------------------------------- #
# ユーティリティ
# --------------------------------------------------------------------------- #


def load_sweep_config(sweep_dir: str) -> dict | None:
    path = os.path.join(sweep_dir, "sweep_config.json")
    if os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    return None


def detect_sweep_type(df: pd.DataFrame) -> tuple[str, list[str]]:
    """スイープの次元を検出する．

    Returns:
        ("1d", [varying_col]) or ("2d", ["features", "traits"])
    """
    n_f = df["features"].nunique()
    n_q = df["traits"].nunique()
    if n_f > 1 and n_q > 1:
        return "2d", ["features", "traits"]
    elif n_f > 1:
        return "1d", ["features"]
    elif n_q > 1:
        return "1d", ["traits"]
    else:
        return "1d", ["features"]


def summarize(df: pd.DataFrame, group_cols: list[str], metric_col: str) -> pd.DataFrame:
    g = df.groupby(group_cols)[metric_col]
    s = g.agg(["mean", "std", "count"]).reset_index()
    s["std"] = s["std"].fillna(0.0)
    s["ci95"] = 1.96 * s["std"] / s["count"].pow(0.5)
    return s


def make_subtitle(config: dict | None, df: pd.DataFrame) -> str:
    parts: list[str] = []
    if config:
        w = config.get("width")
        h = config.get("height")
        if w and h:
            parts.append(f"{w}×{h} グリッド")
    n_runs = df["run"].nunique() if "run" in df.columns else len(df)
    parts.append(f"{n_runs} runs / 条件")
    return "，".join(parts)


# --------------------------------------------------------------------------- #
# 2D ヒートマップ
# --------------------------------------------------------------------------- #


def _heatmap(
    ax: plt.Axes,
    summary: pd.DataFrame,
    value_col: str,
    cmap: str,
    title: str,
    cbar_label: str,
    fmt: str = ".1f",
) -> None:
    pivot = summary.pivot(index="features", columns="traits", values=value_col)
    sns.heatmap(
        pivot,
        annot=True,
        fmt=fmt,
        cmap=cmap,
        ax=ax,
        cbar_kws={"label": cbar_label},
        linewidths=0.5,
        linecolor="#EEEEEE",
    )
    ax.set_title(title)
    ax.set_xlabel("特性数 q")
    ax.set_ylabel("特徴数 f")
    ax.invert_yaxis()


# --------------------------------------------------------------------------- #
# 周辺折れ線プロット
# --------------------------------------------------------------------------- #


def _marginal_lines(
    ax: plt.Axes,
    summary: pd.DataFrame,
    x_col: str,
    group_col: str,
    metric_col: str,
    title: str,
) -> None:
    ax.set_facecolor(COLOR_BG)
    group_vals = sorted(summary[group_col].unique())
    cmap = plt.get_cmap("tab10", max(len(group_vals), 1))
    label_map = {"features": "f", "traits": "q"}
    for i, gv in enumerate(group_vals):
        sub = summary[summary[group_col] == gv].sort_values(x_col)
        ax.errorbar(
            sub[x_col],
            sub["mean"],
            yerr=sub["ci95"],
            fmt="-o",
            capsize=3,
            color=cmap(i),
            label=f"{label_map.get(group_col, group_col)}={gv}",
            alpha=0.9,
        )
    ax.set_xlabel(f"{'特徴数 f' if x_col == 'features' else '特性数 q'}")
    ax.set_ylabel(f"平均 {metric_col}")
    ax.set_title(title)
    ax.grid(True, alpha=0.3)
    ax.legend(fontsize=8, loc="best")


def _marginal_single(
    ax: plt.Axes,
    summary: pd.DataFrame,
    x_col: str,
    metric_col: str,
    title: str,
) -> None:
    ax.set_facecolor(COLOR_BG)
    sub = summary.sort_values(x_col)
    ax.errorbar(
        sub[x_col], sub["mean"], yerr=sub["ci95"],
        fmt="-o", capsize=4, color="#4c97c9",
    )
    ax.set_xlabel(f"{'特徴数 f' if x_col == 'features' else '特性数 q'}")
    ax.set_ylabel(f"平均 {metric_col}")
    ax.set_title(title)
    ax.grid(True, alpha=0.3)


# --------------------------------------------------------------------------- #
# 図の生成
# --------------------------------------------------------------------------- #


def save_heatmaps(summary_2d: pd.DataFrame, out_dir: str, subtitle: str) -> None:
    # 平均
    fig, ax = plt.subplots(figsize=(8, 6), facecolor=COLOR_BG)
    _heatmap(
        ax, summary_2d, "mean", "viridis",
        f"平均 {METRIC_COL}",
        f"平均 {METRIC_COL}",
        fmt=".1f",
    )
    fig.suptitle(f"{METRIC_COL}: f × q", fontsize=13)
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")
    fig.tight_layout(rect=[0, 0, 1, 0.92])
    out = os.path.join(out_dir, "sweep_heatmap_regions.png")
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out}")

    # 95%CI
    fig, ax = plt.subplots(figsize=(8, 6), facecolor=COLOR_BG)
    _heatmap(
        ax, summary_2d, "ci95", "Oranges",
        f"95% CI (f × q)",
        "95% CI",
        fmt=".2f",
    )
    fig.suptitle(f"{METRIC_COL} の 95% 信頼区間", fontsize=13)
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")
    fig.tight_layout(rect=[0, 0, 1, 0.92])
    out = os.path.join(out_dir, "sweep_heatmap_ci.png")
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out}")


def save_marginals_2d(summary_2d: pd.DataFrame, out_dir: str, subtitle: str) -> None:
    fig, ax = plt.subplots(figsize=(9, 5), facecolor=COLOR_BG)
    _marginal_lines(
        ax, summary_2d, "features", "traits", METRIC_COL,
        f"{METRIC_COL} vs 特徴数 f (q ごとの系列)",
    )
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")
    fig.tight_layout(rect=[0, 0, 1, 0.93])
    out = os.path.join(out_dir, "sweep_marginal_features.png")
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out}")

    fig, ax = plt.subplots(figsize=(9, 5), facecolor=COLOR_BG)
    _marginal_lines(
        ax, summary_2d, "traits", "features", METRIC_COL,
        f"{METRIC_COL} vs 特性数 q (f ごとの系列)",
    )
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")
    fig.tight_layout(rect=[0, 0, 1, 0.93])
    out = os.path.join(out_dir, "sweep_marginal_traits.png")
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out}")


def save_marginal_1d(summary_1d: pd.DataFrame, x_col: str, out_dir: str, subtitle: str) -> None:
    fig, ax = plt.subplots(figsize=(8, 5), facecolor=COLOR_BG)
    _marginal_single(
        ax, summary_1d, x_col, METRIC_COL,
        f"{METRIC_COL} vs {'特徴数 f' if x_col == 'features' else '特性数 q'}",
    )
    if subtitle:
        fig.text(0.5, 0.93, subtitle, ha="center", fontsize=9, color="#666666")
    fig.tight_layout(rect=[0, 0, 1, 0.93])
    out = os.path.join(out_dir, f"sweep_marginal_{x_col}.png")
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out}")


def save_overview_2d(summary_2d: pd.DataFrame, out_dir: str, subtitle: str) -> None:
    fig, axes = plt.subplots(2, 2, figsize=(14, 10), facecolor=COLOR_BG)
    fig.suptitle("Axelrod 文化拡散モデル — スイープ概要 (f × q)", fontsize=14)
    if subtitle:
        fig.text(0.5, 0.95, subtitle, ha="center", fontsize=9, color="#666666")

    _heatmap(
        axes[0, 0], summary_2d, "mean", "viridis",
        f"平均 {METRIC_COL}", f"平均 {METRIC_COL}", fmt=".1f",
    )
    _heatmap(
        axes[0, 1], summary_2d, "ci95", "Oranges",
        "95% CI", "95% CI", fmt=".2f",
    )
    _marginal_lines(
        axes[1, 0], summary_2d, "features", "traits", METRIC_COL,
        f"{METRIC_COL} vs 特徴数 f",
    )
    _marginal_lines(
        axes[1, 1], summary_2d, "traits", "features", METRIC_COL,
        f"{METRIC_COL} vs 特性数 q",
    )

    fig.tight_layout(rect=[0, 0, 1, 0.93])
    out = os.path.join(out_dir, "sweep_overview.png")
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  保存: {out}")


# --------------------------------------------------------------------------- #
# メイン
# --------------------------------------------------------------------------- #


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Axelrod 文化拡散モデル パラメータスイープ 可視化スクリプト"
    )
    p.add_argument(
        "--sweep_dir", default="results/latest",
        help="スイープ結果のディレクトリ (default: results/latest)",
    )
    p.add_argument(
        "--output_dir", default=None,
        help="図の保存先ディレクトリ (default: {sweep_dir}/figures)",
    )
    return p.parse_args()


def main() -> None:
    args = parse_args()
    sweep_dir = args.sweep_dir
    out_dir = args.output_dir if args.output_dir else os.path.join(sweep_dir, "figures")

    summary_path = os.path.join(sweep_dir, "sweep_summary.csv")
    if not os.path.exists(summary_path):
        print(f"エラー: sweep_summary.csv が見つかりません: {summary_path}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(out_dir, exist_ok=True)

    print("=== Axelrod 文化拡散モデル スイープ可視化 ===")
    print(f"スイープ結果: {sweep_dir}")
    print(f"出力先:       {out_dir}")
    print("---------------------------------------------")

    df = pd.read_csv(summary_path)
    config = load_sweep_config(sweep_dir)
    sweep_type, sweep_cols = detect_sweep_type(df)
    subtitle = make_subtitle(config, df)

    print(f"スイープ種別: {sweep_type} ({', '.join(sweep_cols)})")
    print(f"{subtitle}")
    print(f"合計 {len(df)} 行")
    print("---------------------------------------------")

    if sweep_type == "2d":
        summary_2d = summarize(df, ["features", "traits"], METRIC_COL)
        save_heatmaps(summary_2d, out_dir, subtitle)
        save_marginals_2d(summary_2d, out_dir, subtitle)
        save_overview_2d(summary_2d, out_dir, subtitle)
    else:
        x_col = sweep_cols[0]
        summary_1d = summarize(df, [x_col], METRIC_COL)
        save_marginal_1d(summary_1d, x_col, out_dir, subtitle)

    print("---------------------------------------------")
    print("完了．出力ファイル一覧:")
    for f in sorted(os.listdir(out_dir)):
        fpath = os.path.join(out_dir, f)
        if os.path.isfile(fpath):
            size_kb = os.path.getsize(fpath) / 1024
            print(f"  {f:40s} ({size_kb:6.1f} KB)")


if __name__ == "__main__":
    main()
