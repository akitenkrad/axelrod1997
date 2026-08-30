#!/usr/bin/env python3
"""runvault_io.py — runvault の run ディレクトリを読むための共通部品．

Rust 側の出力は次の形になっている:

    <results_root>/<experiment>/<run_slug>/
    ├── run.json          ← run のメタデータ (lineage / rng / research …)
    ├── config.json       ← 封筒．実験条件は ["parameters"] の下
    ├── metrics.csv       ← long 形式 (run_uid, step, step_unit, scope, name, value)
    ├── events.jsonl      ← 試行 (観測主体) ごとの 1 行
    ├── status.json
    ├── manifest.csv
    └── artifacts/        ← 実験コードが**実行中に**書いた成果物

このモジュールは «run をどう選ぶか» と «封筒をどう開けるか» を 1 箇所に集める．
このリポジトリには runvault 以前の `results/` が無いので，legacy レイアウトの
分岐は持たない．

注: 同じものが schelling1971 にもある．runvault の Phase 4 (`runvault-py`) が
これを吸収する予定で，それまでは 2 リポジトリで重複させる．
"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
from typing import Iterable, Sequence

import pandas as pd

__all__ = [
    "runvault_path",
    "runvault_binary",
    "config_parameters",
    "load_run_meta",
    "run_subcommand",
    "artifacts_dir",
    "figures_dir",
    "run_scope_metrics",
    "events_table",
    "sweep_children",
    "sweep_summary_table",
    "sweep_events_table",
]


# --------------------------------------------------------------------------- #
# run の選び方
# --------------------------------------------------------------------------- #

def runvault_binary() -> str:
    """`runvault` の実行ファイル．環境変数 `RUNVAULT` があればそれを使う．

    まだ `cargo install` されていない環境が普通なので，PATH だけを頼りにしない．
    """
    override = os.environ.get("RUNVAULT")
    if override:
        if not os.path.isfile(override) or not os.access(override, os.X_OK):
            raise SystemExit(
                f"エラー: 環境変数 RUNVAULT が実行可能なファイルを指していません: {override}"
            )
        return override
    found = shutil.which("runvault")
    if found is None:
        raise SystemExit(
            "エラー: `runvault` コマンドが見つかりません．\n"
            "  次のいずれかにしてください．\n"
            "    - PATH に入れる: cargo install --path <rs-runvault>/crates/runvault\n"
            "    - 実行ファイルを指す: RUNVAULT=<rs-runvault>/target/debug/runvault ...\n"
            "    - run ディレクトリを直接渡す: --results_dir <run>"
        )
    return found


def _runvault_paths(args: list[str], what: str) -> list[str]:
    """`runvault path` を呼んで結果の行を返す．"""
    cmd = [runvault_binary(), "path"] + args
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        raise SystemExit(
            f"エラー: {what}が見つかりません ({' '.join(cmd)})\n"
            f"  {proc.stderr.strip()}"
        )
    lines = [line for line in proc.stdout.strip().splitlines() if line]
    if not lines:
        raise SystemExit(f"エラー: runvault が空の結果を返しました ({' '.join(cmd)})")
    return lines


def runvault_path(
    experiment: str,
    results_root: str = "results",
    subcommand: str | None = None,
    standalone: bool = False,
) -> str:
    """`runvault path --latest` で直近の完了 run のパスを得る．

    `subcommand` を渡すと，その subcommand の run だけが対象になる．sweep の親と
    子は同じ experiment に居るため，これを付けないと `--latest` が指標を持たない
    親を返すことがある．

    `standalone=True` を渡すと，さらに sweep に属さない run だけになる．sweep の子は
    手で起こした run と subcommand が同じなので，`--subcommand simulate` だけでは
    «最後に走った子» が返る．単体の run を見たいときは必ず付ける．
    """
    args = [
        "--results-root", str(results_root),
        "--experiment", experiment,
        "--latest",
    ]
    if subcommand is not None:
        args += ["--subcommand", subcommand]
    if standalone:
        args += ["--standalone"]
    return _runvault_paths(args, "完了した run")[0]


# --------------------------------------------------------------------------- #
# 封筒を開ける
# --------------------------------------------------------------------------- #

def config_parameters(run_dir: str | os.PathLike) -> dict:
    """run ディレクトリの `config.json` から実験条件を取り出す．

    runvault の config.json は `{schema_version, run_uid, runvault, parameters}`
    という封筒で，条件は `parameters` の下にある．
    """
    path = os.path.join(str(run_dir), "config.json")
    if not os.path.exists(path):
        raise FileNotFoundError(f"config.json が見つかりません: {path}")
    with open(path) as f:
        doc = json.load(f)
    return doc["parameters"]


def load_run_meta(run_dir: str | os.PathLike) -> dict:
    """`run.json` を読む．"""
    path = os.path.join(str(run_dir), "run.json")
    if not os.path.exists(path):
        raise FileNotFoundError(
            f"run.json が見つかりません (runvault の run ディレクトリではない): {path}"
        )
    with open(path) as f:
        return json.load(f)


def run_subcommand(run_dir: str | os.PathLike) -> str:
    """この run がどのサブコマンドの実行か (`simulate` / `sweep`)．"""
    return str(load_run_meta(run_dir)["subcommand"])


def artifacts_dir(run_dir: str | os.PathLike) -> str:
    """**実行中に**実験コードが書いた出力の置き場．

    `manifest.csv` は `finish()` が `artifacts/` と `logs/` を歩いて確定させるので，
    ここに入るのは run が終わるまでに書かれたものだけである．**後から作る図を
    ここに置いてはいけない** (ハッシュが付かず，run の記録でもない)．
    作図の出力先は [`figures_dir`]．
    """
    return os.path.join(str(run_dir), "artifacts")


def figures_dir(run_dir: str | os.PathLike) -> str:
    """作図の出力先．run が終わった後に作るものは run の記録ではない．

    `<results_root>/<experiment>/figures/<run_slug>/` を返す．run ディレクトリの
    外に置くので，`finish()` が確定させた `manifest.csv` と食い違わない．
    """
    run_dir = os.path.abspath(str(run_dir))
    experiment_dir = os.path.dirname(run_dir)
    return os.path.join(experiment_dir, "figures", os.path.basename(run_dir))


# --------------------------------------------------------------------------- #
# metrics.csv — run 全体の集約
# --------------------------------------------------------------------------- #

def run_scope_metrics(run_dir: str | os.PathLike) -> dict[str, float]:
    """run 全体を 1 つの値で表す指標 (`n_units` / `convergence_rate` など)．

    long 形式の `metrics.csv` のうち step を持たない行を集める．
    """
    path = os.path.join(str(run_dir), "metrics.csv")
    if not os.path.exists(path):
        return {}
    df = pd.read_csv(path)
    if df.empty:
        return {}
    rows = df[df["step"].isna()]
    return {str(r["name"]): float(r["value"]) for _, r in rows.iterrows()}


# --------------------------------------------------------------------------- #
# events.jsonl — 観測主体 (試行) ごとの記録
# --------------------------------------------------------------------------- #

def events_table(run_dir: str | os.PathLike, kind: str = "terminal") -> pd.DataFrame:
    """`events.jsonl` を DataFrame にする．

    `kind` で `schema` 列を絞る (`None` なら全種)．予約語 (`unit_id` / `t` /
    `t_unit` / `outcome` / `censored` / `budget`) はそのままの名前で列になり，
    実験固有のフィールドも同じ行に並ぶ．
    """
    path = os.path.join(str(run_dir), "events.jsonl")
    if not os.path.exists(path):
        raise FileNotFoundError(f"events.jsonl が見つかりません: {path}")
    rows: list[dict] = []
    with open(path) as f:
        for line in f:
            if not line.strip():
                continue
            row = json.loads(line)
            if kind is not None and row.get("schema") != kind:
                continue
            rows.append(row)
    if not rows:
        raise SystemExit(
            f"エラー: {path} に schema={kind} のイベントがありません．"
        )
    return pd.DataFrame(rows)


# --------------------------------------------------------------------------- #
# sweep の系譜
# --------------------------------------------------------------------------- #

def sweep_children(parent_dir: str | os.PathLike) -> list[str]:
    """sweep 親 run の子 run を集める．

    子は親の下ではなく experiment ディレクトリの兄弟として並ぶ．系譜の突き合わせは
    `runvault path --children-of <parent run_uid>` に任せる (自前で
    `lineage.parent_run_uid` を走査しない)．返り値はディレクトリ名の昇順
    (＝開始時刻の昇順)．
    """
    parent = os.path.abspath(str(parent_dir))
    meta = load_run_meta(parent)
    experiment_dir = os.path.dirname(parent)
    results_root = os.path.dirname(experiment_dir)
    children = _runvault_paths(
        [
            "--results-root", results_root,
            "--experiment", meta["experiment"],
            "--children-of", meta["run_uid"],
        ],
        f"sweep 親 {meta['run_slug']} の子 run",
    )
    return sorted(children, key=os.path.basename)


def sweep_summary_table(
    sweep_dir: str | os.PathLike,
    parameter_keys: Sequence[str],
    metric_names: Iterable[str] | None = None,
) -> pd.DataFrame:
    """1 行 1 条件のサマリ表を用意する．

    runvault ではこの表はファイルとして存在しない．sweep 親の子 run を集め，
    各子の `config.json` の `parameters` と `metrics.csv` の run スコープ指標から
    組み直す．

    Args:
        sweep_dir: sweep 親 run のディレクトリ．
        parameter_keys: 列にしたい `/parameters` のキー (例 `["features", "traits"]`)．
            この順に並べ，最後にソートキーとしても使う．
        metric_names: 列にしたい run スコープ指標名．`None` なら子が持つものを全部．

    どの経路でも `run_dir` 列を付けるので，呼び出し側は条件からディレクトリ名を
    組み立てなくてよい．
    """
    rows: list[dict] = []
    for child in sweep_children(sweep_dir):
        params = config_parameters(child)
        scoped = run_scope_metrics(child)
        wanted = list(scoped) if metric_names is None else list(metric_names)
        row: dict = {key: params.get(key) for key in parameter_keys}
        row.update({name: scoped.get(name) for name in wanted})
        row["run_dir"] = child
        rows.append(row)
    return (
        pd.DataFrame(rows)
        .sort_values(list(parameter_keys))
        .reset_index(drop=True)
    )


def sweep_events_table(
    sweep_dir: str | os.PathLike,
    parameter_keys: Sequence[str],
    kind: str = "terminal",
) -> pd.DataFrame:
    """1 行 1 試行の表を用意する (条件の列を付けた `events_table` の連結)．

    条件ごとの平均だけでなく分散・信頼区間を出すには試行ごとの値が要るが，それは
    集約指標ではなく `events.jsonl` の担当なので，子 run のイベントを縦に積む．
    """
    frames: list[pd.DataFrame] = []
    for child in sweep_children(sweep_dir):
        params = config_parameters(child)
        df = events_table(child, kind=kind)
        for key in parameter_keys:
            df[key] = params.get(key)
        df["run_dir"] = child
        frames.append(df)
    return (
        pd.concat(frames, ignore_index=True)
        .sort_values(list(parameter_keys) + ["unit_id"])
        .reset_index(drop=True)
    )
