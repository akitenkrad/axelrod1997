#!/usr/bin/env python3
"""
visualize.py — Axelrod (1997) 文化拡散モデル 可視化スクリプト（simulate / sweep 両対応）

`config.json` の `"subcommand"` フィールドに応じて，simulate モードの分布・
指標サマリ・Table 7-2 比較プロット，または sweep モードのヒートマップ・周辺
折れ線・概要パネルを自動生成する．`config.json` が無い場合はディレクトリ名の
接頭辞（`simulate_` / `sweep_`）からサブコマンドを推定する．

Usage:
    uv run python analysis/visualize.py
    uv run python analysis/visualize.py --results_dir results/simulate_20260417_120000
    uv run python analysis/visualize.py --results_dir results/latest --output_dir out
"""

from __future__ import annotations

import argparse
import json
import os
import sys

import matplotlib.cm as cm  # noqa: F401  (互換性のため残す)
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns

# --------------------------------------------------------------------------- #
# 日本語フォント設定・カラーパレット
# --------------------------------------------------------------------------- #
plt.rcParams["font.family"] = "Hiragino Sans"

COLOR_BG = "#FAFAF8"
COLOR_MAIN = "#4c97c9"
COLOR_ACCENT = "#F44336"
COLOR_REF = "#9C27B0"

METRIC_COL = "n_stable_regions"

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
# 共通ユーティリティ
# --------------------------------------------------------------------------- #


def load_config(results_dir: str) -> dict | None:
    path = os.path.join(results_dir, "config.json")
    if os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    return None


def detect_subcommand(results_dir: str, config: dict | None) -> str:
    """`config.json` の `subcommand` か，ディレクトリ名の接頭辞からサブコマンドを判定する．"""
    if config and "subcommand" in config:
        return str(config["subcommand"])
    base = os.path.basename(os.path.normpath(results_dir))
    if base.startswith("sweep"):
        return "sweep"
    return "simulate"


# --------------------------------------------------------------------------- #
# simulate モード
# --------------------------------------------------------------------------- #


def _summarize_simulate(df: pd.DataFrame) -> dict:
    """各指標の平均と95%CIを計算する"""
    out: dict = {}
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


def _plot_simulate_distribution(df: pd.DataFrame, out_path: str, subtitle: str) -> None:
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


def _plot_simulate_metrics(summary: dict, out_path: str, subtitle: str) -> None:
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


def _plot_simulate_vs_table_7_2(df: pd.DataFrame, config: dict | None, out_path: str) -> bool:
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


def run_simulate_mode(df: pd.DataFrame, config: dict | None, out_dir: str) -> None:
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

    summary = _summarize_simulate(df)

    print("[1/3] 分布プロットを保存中 ...")
    _plot_simulate_distribution(df, os.path.join(out_dir, "simulate_distribution.png"), subtitle)

    print("[2/3] 指標サマリを保存中 ...")
    _plot_simulate_metrics(summary, os.path.join(out_dir, "simulate_metrics.png"), subtitle)

    print("[3/3] Table 7-2 との比較を保存中 ...")
    saved = _plot_simulate_vs_table_7_2(
        df, config, os.path.join(out_dir, "simulate_vs_table7_2.png")
    )
    if not saved:
        print("  (f, q) が Table 7-2 ベンチマーク外のためスキップ")

    # サマリ表示
    print("-----------------------------------------------")
    print("指標サマリ:")
    for col, s in summary.items():
        print(f"  {col:<22} mean={s['mean']:.3f}  95%CI=±{s['ci95']:.3f}  n={s['n']}")


# --------------------------------------------------------------------------- #
# sweep モード
# --------------------------------------------------------------------------- #


def _detect_sweep_type(df: pd.DataFrame) -> tuple[str, list[str]]:
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


def _summarize_sweep(df: pd.DataFrame, group_cols: list[str], metric_col: str) -> pd.DataFrame:
    g = df.groupby(group_cols)[metric_col]
    s = g.agg(["mean", "std", "count"]).reset_index()
    s["std"] = s["std"].fillna(0.0)
    s["ci95"] = 1.96 * s["std"] / s["count"].pow(0.5)
    return s


def _make_sweep_subtitle(config: dict | None, df: pd.DataFrame) -> str:
    parts: list[str] = []
    if config:
        w = config.get("width")
        h = config.get("height")
        if w and h:
            parts.append(f"{w}×{h} グリッド")
    n_runs = df["run"].nunique() if "run" in df.columns else len(df)
    parts.append(f"{n_runs} runs / 条件")
    return "，".join(parts)


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
        fmt="-o", capsize=4, color=COLOR_MAIN,
    )
    ax.set_xlabel(f"{'特徴数 f' if x_col == 'features' else '特性数 q'}")
    ax.set_ylabel(f"平均 {metric_col}")
    ax.set_title(title)
    ax.grid(True, alpha=0.3)


def _save_sweep_heatmaps(summary_2d: pd.DataFrame, out_dir: str, subtitle: str) -> None:
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
        "95% CI (f × q)",
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


def _save_sweep_marginals_2d(summary_2d: pd.DataFrame, out_dir: str, subtitle: str) -> None:
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


def _save_sweep_marginal_1d(summary_1d: pd.DataFrame, x_col: str, out_dir: str, subtitle: str) -> None:
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


def _save_sweep_overview_2d(summary_2d: pd.DataFrame, out_dir: str, subtitle: str) -> None:
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


def run_sweep_mode(df: pd.DataFrame, config: dict | None, out_dir: str) -> None:
    sweep_type, sweep_cols = _detect_sweep_type(df)
    subtitle = _make_sweep_subtitle(config, df)

    print(f"スイープ種別: {sweep_type} ({', '.join(sweep_cols)})")
    print(f"{subtitle}")
    print(f"合計 {len(df)} 行")
    print("---------------------------------------------")

    if sweep_type == "2d":
        summary_2d = _summarize_sweep(df, ["features", "traits"], METRIC_COL)
        _save_sweep_heatmaps(summary_2d, out_dir, subtitle)
        _save_sweep_marginals_2d(summary_2d, out_dir, subtitle)
        _save_sweep_overview_2d(summary_2d, out_dir, subtitle)
    else:
        x_col = sweep_cols[0]
        summary_1d = _summarize_sweep(df, [x_col], METRIC_COL)
        _save_sweep_marginal_1d(summary_1d, x_col, out_dir, subtitle)


# --------------------------------------------------------------------------- #
# メイン
# --------------------------------------------------------------------------- #


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Axelrod 文化拡散モデル 可視化スクリプト（simulate / sweep 両対応）"
    )
    p.add_argument(
        "--results_dir", default="results/latest",
        help="結果ディレクトリ (default: results/latest)",
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

    config = load_config(results_dir)
    subcmd = detect_subcommand(results_dir, config)

    print("=== Axelrod 文化拡散モデル 可視化 ===")
    print(f"結果ディレクトリ: {results_dir}")
    print(f"サブコマンド:     {subcmd}")
    print(f"出力先:           {out_dir}")
    print("-----------------------------------------------")

    df = pd.read_csv(metrics_path)

    if subcmd == "sweep":
        run_sweep_mode(df, config, out_dir)
    else:
        run_simulate_mode(df, config, out_dir)

    print("-----------------------------------------------")
    print("完了．出力ファイル一覧:")
    for f in sorted(os.listdir(out_dir)):
        fpath = os.path.join(out_dir, f)
        if os.path.isfile(fpath):
            size_kb = os.path.getsize(fpath) / 1024
            print(f"  {f:40s} ({size_kb:6.1f} KB)")


if __name__ == "__main__":
    main()
