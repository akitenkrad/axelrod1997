use axelrod_culture::{metrics, record, simulation};

use clap::{Parser, Subcommand};
use runvault::{Lineage, Run, RunOptions, Stage};
use serde::Serialize;

use metrics::count_stable_regions;
use record::{Trial, DOMAIN, EXPERIMENT, REPO_ID};

// ---------------------------------------------------------------------------
// CLI 定義
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "axelrod",
    about = "Axelrod (1997) The Dissemination of Culture — 再現実験"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 単一パラメータ設定で複数試行を実行する
    Simulate(SimulateArgs),
    /// 特徴数 f × 特性数 q の感度分析（グリッドサーチ）を実行する
    Sweep(SweepArgs),
}

#[derive(Parser, Debug)]
struct SimulateArgs {
    /// グリッド幅
    #[arg(long, default_value_t = 10)]
    width: usize,

    /// グリッド高さ
    #[arg(long, default_value_t = 10)]
    height: usize,

    /// 特徴数 f
    #[arg(long, short = 'f', default_value_t = 5)]
    features: usize,

    /// 特性数 q
    #[arg(long, short = 'q', default_value_t = 10)]
    traits: usize,

    /// 試行回数
    #[arg(long, default_value_t = 10)]
    runs: usize,

    /// 1試行あたりの最大イベント数
    #[arg(long, default_value_t = 1_000_000)]
    max_events: usize,

    /// 乱数シード（省略時はランダムに実体化し，run の master_seed として記録する）
    #[arg(long)]
    seed: Option<u64>,

    /// results ルート（この下に <experiment>/<run_slug>/ ができる）
    #[arg(long, default_value = "results")]
    output_dir: String,
}

#[derive(Parser, Debug)]
struct SweepArgs {
    /// グリッド幅
    #[arg(long, default_value_t = 10)]
    width: usize,

    /// グリッド高さ
    #[arg(long, default_value_t = 10)]
    height: usize,

    /// 特徴数 f の開始値
    #[arg(long, default_value_t = 5)]
    features_min: usize,

    /// 特徴数 f の終了値（含む）
    #[arg(long, default_value_t = 15)]
    features_max: usize,

    /// 特徴数 f の刻み幅
    #[arg(long, default_value_t = 5)]
    features_step: usize,

    /// 特性数 q の開始値
    #[arg(long, default_value_t = 5)]
    traits_min: usize,

    /// 特性数 q の終了値（含む）
    #[arg(long, default_value_t = 15)]
    traits_max: usize,

    /// 特性数 q の刻み幅
    #[arg(long, default_value_t = 5)]
    traits_step: usize,

    /// 各 (f, q) 組み合わせあたりの試行回数
    #[arg(long, default_value_t = 10)]
    runs: usize,

    /// 1試行あたりの最大イベント数
    #[arg(long, default_value_t = 1_000_000)]
    max_events: usize,

    /// 乱数シード（省略時はランダムに実体化し，各子 run の master_seed として記録する）
    #[arg(long)]
    seed: Option<u64>,

    /// results ルート（この下に <experiment>/<run_slug>/ ができる）
    #[arg(long, default_value = "results")]
    output_dir: String,
}

// ---------------------------------------------------------------------------
// /parameters に書く実験条件
//
// 条件だけを持つ．出力先は run ディレクトリそのものなので持たない．
// 計算結果 (収束率・地域数) は指標であって条件ではないので入れない．
// ---------------------------------------------------------------------------

/// `simulate` の 1 条件．`sweep` の子 run も同じ形を持つ．
#[derive(Serialize, Debug, Clone)]
struct SimulateParameters {
    width: usize,
    height: usize,
    features: usize,
    traits: usize,
    runs: usize,
    max_events: usize,
    /// 実体化済みの base seed．試行ごとのシードはここから決定的に派生する．
    seed: u64,
}

/// `sweep` 親のグリッド定義．
#[derive(Serialize, Debug, Clone)]
struct SweepParameters {
    width: usize,
    height: usize,
    features: AxisRange,
    traits: AxisRange,
    runs: usize,
    max_events: usize,
    seed: u64,
}

#[derive(Serialize, Debug, Clone)]
struct AxisRange {
    min: usize,
    max: usize,
    step: usize,
}

impl AxisRange {
    /// 端点を含む等差列に展開する．`step` は 1 未満にならないよう丸める．
    fn values(&self) -> Vec<usize> {
        let step = self.step.max(1);
        let mut out = Vec::new();
        let mut v = self.min;
        while v <= self.max {
            out.push(v);
            v += step;
        }
        out
    }
}

// ---------------------------------------------------------------------------
// 1 条件の実行
// ---------------------------------------------------------------------------

/// 1 条件を `runs` 回試行し，各試行を `terminal` イベントとして記録する．
///
/// 試行は子 run にしない．1 つの run の中の観測主体 (`unit_id = trial-N`) として
/// 扱うので，生存時間解析はこの run の `events.jsonl` だけで組める．
///
/// 進捗の 1 単位は 1 試行．`stage` は呼び出し側が開ける — `simulate` では 1 条件の
/// `runs` 試行が，`sweep` ではグリッド全体の試行が 1 つの stage になり，後者は
/// 条件をまたいでも割合が途中で 100% に戻らない．
fn run_condition(
    rv: &mut Run,
    p: &SimulateParameters,
    verbose: bool,
    stage: &mut Stage,
) -> Vec<Trial> {
    let mut trials = Vec::with_capacity(p.runs);

    for index in 0..p.runs {
        let seed = record::trial_seed(p.seed, p.features, p.traits, index);
        let result = simulation::run(
            p.width,
            p.height,
            p.features,
            p.traits,
            p.max_events,
            seed,
        );
        let m = count_stable_regions(&result.world);

        let trial = Trial {
            index,
            seed,
            converged: result.converged,
            n_events: result.n_events,
            max_events: p.max_events,
            metrics: m,
        };

        if verbose {
            println!(
                "[{}/{}] seed={:>20} converged={:<5} events={:>10} regions={:>3} max_region={:>3} distinct={:>3}",
                index + 1,
                p.runs,
                trial.seed,
                trial.converged,
                trial.n_events,
                trial.metrics.n_stable_regions,
                trial.metrics.max_region_size,
                trial.metrics.n_distinct_cultures,
            );
        }

        record::log_trial(rv, &trial);
        trials.push(trial);
        // 収束しなかった試行も数える．数えているのは «試みた仕事» であって
        // «うまくいった仕事» ではない．
        stage.tick();
    }

    record::log_run_summary(rv, &trials);
    trials
}

fn n_converged(trials: &[Trial]) -> usize {
    trials.iter().filter(|t| t.converged).count()
}

fn mean_regions(trials: &[Trial]) -> f64 {
    if trials.is_empty() {
        return f64::NAN;
    }
    trials
        .iter()
        .map(|t| t.metrics.n_stable_regions as f64)
        .sum::<f64>()
        / trials.len() as f64
}

// ---------------------------------------------------------------------------
// simulate サブコマンド
// ---------------------------------------------------------------------------

fn cmd_simulate(args: SimulateArgs) {
    // シードを実体化してから記録する．--seed 省略時に試行側で rand::random に
    // 落とすと，実際に使われたシードがどこにも残らない．
    let seed = args.seed.unwrap_or_else(rand::random::<u64>);

    let params = SimulateParameters {
        width: args.width,
        height: args.height,
        features: args.features,
        traits: args.traits,
        runs: args.runs,
        max_events: args.max_events,
        seed,
    };

    let mut rv = Run::start(
        RunOptions::new(EXPERIMENT, "simulate")
            .repo_id(REPO_ID)
            .domain(DOMAIN)
            .results_root(&args.output_dir)
            .parameters(&params)
            .expect("runvault: parameters の組み立てに失敗")
            .seed_pointers(["/seed"])
            .master_seed(seed)
            .replication(record::replication()),
    )
    .expect("runvault: run の開始に失敗");

    println!("=== Axelrod 文化拡散モデル 再現実験 ===");
    println!(
        "グリッド: {}×{} | f={} | q={} | runs={} | max_events={}",
        params.width, params.height, params.features, params.traits, params.runs, params.max_events
    );
    println!("シード (base): {}", seed);
    println!("出力先: {}", rv.dir().display());
    println!("---------------------------------------");

    // 1 試行 = 1 単位．試行ごとの費用は max_events で頭打ちになり，収束する条件
    // でも実測で 2 倍の幅しかなかった (10x10 の既定グリッドで 0.70M〜1.42M
    // イベント) ので，重み付けではなく数える．そもそも 1 試行が何イベントで
    // 終わるかは走らせるまで分からず，測っていない費用モデルは自信をもって
    // 外れた見積もりを出す．
    let mut stage = rv.stage("trials", params.runs);
    let trials = run_condition(&mut rv, &params, true, &mut stage);
    stage.close();

    println!("---------------------------------------");
    println!(
        "完了: {}/{} が収束 | 平均 n_stable_regions = {:.2}",
        n_converged(&trials),
        trials.len(),
        mean_regions(&trials)
    );

    let dir = rv.finish().expect("runvault: run の完了に失敗");
    println!("設定       → {}/config.json", dir.display());
    println!("集約指標   → {}/metrics.csv", dir.display());
    println!("試行ごと   → {}/events.jsonl", dir.display());
}

// ---------------------------------------------------------------------------
// sweep サブコマンド
// ---------------------------------------------------------------------------

fn cmd_sweep(args: SweepArgs) {
    let seed = args.seed.unwrap_or_else(rand::random::<u64>);

    let sweep_params = SweepParameters {
        width: args.width,
        height: args.height,
        features: AxisRange {
            min: args.features_min,
            max: args.features_max,
            step: args.features_step,
        },
        traits: AxisRange {
            min: args.traits_min,
            max: args.traits_max,
            step: args.traits_step,
        },
        runs: args.runs,
        max_events: args.max_events,
        seed,
    };

    let feature_vals = sweep_params.features.values();
    let traits_vals = sweep_params.traits.values();
    assert!(
        !feature_vals.is_empty() && !traits_vals.is_empty(),
        "探索するグリッドが空です (features_min > features_max か traits_min > traits_max)"
    );

    let n_combos = feature_vals.len() * traits_vals.len();
    let n_total = n_combos * sweep_params.runs;

    // 親 run: グリッド定義そのものを parameters に持つ．個別条件の指標は書かない．
    // 親は単一の master_seed を持たない (条件ごとの子がそれぞれ持つ)．base seed は
    // /parameters.seed と seed_pointers 経由で execution_hash に残る．
    // sweep_id は runvault が親の run_slug で埋める．
    let parent = Run::start(
        RunOptions::new(EXPERIMENT, "sweep")
            .repo_id(REPO_ID)
            .domain(DOMAIN)
            .results_root(&args.output_dir)
            .parameters(&sweep_params)
            .expect("runvault: sweep の parameters の組み立てに失敗")
            .seed_pointers(["/seed"])
            .sweep_parent()
            .replication(record::replication()),
    )
    .expect("runvault: sweep 親 run の開始に失敗");

    let sweep_id = parent
        .sweep_id()
        .expect("runvault: sweep 親に sweep_id がありません")
        .to_string();
    let parent_run_uid = parent.run_uid().to_string();

    println!("=== Axelrod 文化拡散モデル パラメータスイープ ===");
    println!(
        "グリッド: {}×{} | f={:?} | q={:?} | runs={} | max_events={}",
        sweep_params.width,
        sweep_params.height,
        feature_vals,
        traits_vals,
        sweep_params.runs,
        sweep_params.max_events
    );
    println!("シード (base): {}", seed);
    println!("合計 {} 試行 ({} 条件 × {} runs)", n_total, n_combos, sweep_params.runs);
    println!("出力先: {}", parent.dir().display());
    println!("-----------------------------------------------");

    // グリッド全体で 1 つの stage．条件ごとに開け直すと 9 個の小さな 100% が
    // 並ぶだけで，スイープ全体のどこにいるかは分からない．単位は条件ではなく
    // 試行 — 条件は既定でも 9 個しかなく，5% 刻みでは 1 条件ごとにしか報告
    // されない．
    let mut stage = parent.stage("trials", n_total);

    let mut idx = 0usize;
    for &features in &feature_vals {
        for &traits in &traits_vals {
            idx += 1;

            let params = SimulateParameters {
                width: sweep_params.width,
                height: sweep_params.height,
                features,
                traits,
                runs: sweep_params.runs,
                max_events: sweep_params.max_events,
                seed,
            };

            // 子は「その条件の simulate」そのもの．master_seed は親と同じ base で，
            // 条件が違えば config_hash が違うので run としては別物になる．
            // 同じ条件の繰り返しは無いので replicate_index は 0．
            let mut child = Run::start(
                RunOptions::new(EXPERIMENT, "simulate")
                    .repo_id(REPO_ID)
                    .domain(DOMAIN)
                    .results_root(&args.output_dir)
                    .parameters(&params)
                    .expect("runvault: 子 run の parameters の組み立てに失敗")
                    .seed_pointers(["/seed"])
                    .master_seed(seed)
                    .replicate_index(0)
                    .lineage(Lineage {
                        sweep_id: Some(sweep_id.clone()),
                        parent_run_uid: Some(parent_run_uid.clone()),
                        ..Default::default()
                    })
                    .replication(record::replication()),
            )
            .expect("runvault: 子 run の開始に失敗");

            let trials = run_condition(&mut child, &params, false, &mut stage);

            println!(
                "[{}/{}] f={:<3} q={:<3} → converged={}/{} mean_regions={:.2}",
                idx,
                n_combos,
                features,
                traits,
                n_converged(&trials),
                trials.len(),
                mean_regions(&trials),
            );

            child.finish().expect("runvault: 子 run の完了に失敗");
        }
    }

    // manifest.csv は finish() で書かれる．その後に progress.log へ 1 行でも
    // 足すと，manifest と食い違うダイジェストになる．
    stage.close();

    let dir = parent.finish().expect("runvault: sweep 親 run の完了に失敗");
    println!("-----------------------------------------------");
    println!("スイープ完了．");
    println!("スイープ定義 → {}/config.json", dir.display());
    println!("各条件の結果は子 run (subcommand=simulate) の events.jsonl / metrics.csv にあります");
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Simulate(args) => cmd_simulate(args),
        Commands::Sweep(args) => cmd_sweep(args),
    }
}

#[cfg(test)]
mod tests {
    use super::AxisRange;

    #[test]
    fn axis_range_includes_both_ends() {
        let r = AxisRange { min: 5, max: 15, step: 5 };
        assert_eq!(r.values(), vec![5, 10, 15]);
    }

    #[test]
    fn axis_range_stops_before_overshooting_max() {
        let r = AxisRange { min: 5, max: 14, step: 5 };
        assert_eq!(r.values(), vec![5, 10]);
    }

    #[test]
    fn axis_range_with_zero_step_does_not_loop_forever() {
        let r = AxisRange { min: 3, max: 3, step: 0 };
        assert_eq!(r.values(), vec![3]);
    }

    #[test]
    fn axis_range_empty_when_min_exceeds_max() {
        let r = AxisRange { min: 10, max: 5, step: 1 };
        assert!(r.values().is_empty());
    }
}
