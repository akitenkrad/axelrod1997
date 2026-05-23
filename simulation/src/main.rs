mod config;
mod mechanisms;
mod metrics;
mod simulation;
mod world;

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use chrono::Local;
use clap::{Parser, Subcommand};
use csv::Writer;

use socsim_core::derive_seed as socsim_derive_seed;

use config::{SimulateConfig, SweepConfig};
use metrics::count_stable_regions;

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

    /// 乱数シード（省略時はランダム．指定すると各 run で決定的に派生）
    #[arg(long)]
    seed: Option<u64>,

    /// 結果出力ディレクトリ
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

    /// 乱数シード（省略時はランダム）
    #[arg(long)]
    seed: Option<u64>,

    /// 結果出力ベースディレクトリ
    #[arg(long, default_value = "results")]
    output_dir: String,
}

// ---------------------------------------------------------------------------
// CSV 出力用の行構造体
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct SimulateRow {
    run: usize,
    width: usize,
    height: usize,
    features: usize,
    traits: usize,
    seed: u64,
    converged: bool,
    n_events: usize,
    n_stable_regions: usize,
    max_region_size: usize,
    n_distinct_cultures: usize,
}

#[derive(serde::Serialize)]
struct SweepRow {
    features: usize,
    traits: usize,
    run: usize,
    width: usize,
    height: usize,
    seed: u64,
    converged: bool,
    n_events: usize,
    n_stable_regions: usize,
    max_region_size: usize,
    n_distinct_cultures: usize,
}

// ---------------------------------------------------------------------------
// ユーティリティ
// ---------------------------------------------------------------------------

/// 単一試行を実行してメトリクスを返す
fn execute_run(
    width: usize,
    height: usize,
    features: usize,
    traits: usize,
    max_events: usize,
    seed: u64,
) -> (bool, usize, metrics::RunMetrics) {
    let result = simulation::run(width, height, features, traits, max_events, seed);
    let m = count_stable_regions(&result.world);
    (result.converged, result.n_events, m)
}

/// 派生シードを作成する．seed が None のときはランダムに生成．
/// 指定時は socsim の決定論的シード派生 `derive_seed(base, &[features, traits, run])` を使う．
fn derive_seed(base: Option<u64>, features: usize, traits: usize, run: usize) -> u64 {
    match base {
        Some(s) => socsim_derive_seed(s, &[features as u64, traits as u64, run as u64]),
        None => rand::random::<u64>(),
    }
}

/// latest シンボリックリンクを更新する（Unix のみ）
fn update_latest_symlink(base_dir: &Path, target_name: &str) {
    let symlink_path = base_dir.join("latest");
    if symlink_path.is_symlink() || symlink_path.exists() {
        let _ = fs::remove_file(&symlink_path);
    }
    #[cfg(unix)]
    {
        let _ = std::os::unix::fs::symlink(target_name, &symlink_path);
    }
}

// ---------------------------------------------------------------------------
// simulate サブコマンド
// ---------------------------------------------------------------------------

fn cmd_simulate(args: SimulateArgs) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_name = timestamp.clone();
    let out_dir = format!("{}/{}", args.output_dir, run_name);
    fs::create_dir_all(&out_dir).expect("出力ディレクトリの作成に失敗");

    let cfg = SimulateConfig {
        width: args.width,
        height: args.height,
        features: args.features,
        traits: args.traits,
        runs: args.runs,
        max_events: args.max_events,
        seed: args.seed,
        output_dir: args.output_dir.clone(),
    };

    println!("=== Axelrod 文化拡散モデル 再現実験 ===");
    println!(
        "グリッド: {}×{} | f={} | q={} | runs={} | max_events={}",
        cfg.width, cfg.height, cfg.features, cfg.traits, cfg.runs, cfg.max_events
    );
    println!("シード (base): {:?}", cfg.seed);
    println!("出力先: {}", out_dir);
    println!("---------------------------------------");

    // config.json を出力
    let config_json = serde_json::json!({
        "subcommand": "simulate",
        "width": cfg.width,
        "height": cfg.height,
        "features": cfg.features,
        "traits": cfg.traits,
        "runs": cfg.runs,
        "max_events": cfg.max_events,
        "seed": cfg.seed,
    });
    let config_path = format!("{}/config.json", out_dir);
    let file = File::create(&config_path).expect("config.json の作成に失敗");
    serde_json::to_writer_pretty(BufWriter::new(file), &config_json)
        .expect("config.json の書き込みに失敗");

    // metrics.csv を書き込む
    let metrics_path = format!("{}/metrics.csv", out_dir);
    let file = File::create(&metrics_path).expect("metrics.csv の作成に失敗");
    let mut wtr = Writer::from_writer(BufWriter::new(file));

    let mut sum_regions = 0.0f64;
    let mut n_converged = 0usize;

    for run in 0..cfg.runs {
        let seed = derive_seed(cfg.seed, cfg.features, cfg.traits, run);
        let (converged, n_events, m) = execute_run(
            cfg.width,
            cfg.height,
            cfg.features,
            cfg.traits,
            cfg.max_events,
            seed,
        );

        if converged {
            n_converged += 1;
        }
        sum_regions += m.n_stable_regions as f64;

        println!(
            "[{}/{}] seed={:>20} converged={:<5} events={:>10} regions={:>3} max_region={:>3} distinct={:>3}",
            run + 1,
            cfg.runs,
            seed,
            converged,
            n_events,
            m.n_stable_regions,
            m.max_region_size,
            m.n_distinct_cultures,
        );

        let row = SimulateRow {
            run,
            width: cfg.width,
            height: cfg.height,
            features: cfg.features,
            traits: cfg.traits,
            seed,
            converged,
            n_events,
            n_stable_regions: m.n_stable_regions,
            max_region_size: m.max_region_size,
            n_distinct_cultures: m.n_distinct_cultures,
        };
        wtr.serialize(row).expect("メトリクス行の書き込みに失敗");
    }
    wtr.flush().expect("フラッシュに失敗");

    // latest シンボリックリンクを更新
    update_latest_symlink(Path::new(&cfg.output_dir), &run_name);

    let mean_regions = sum_regions / (cfg.runs as f64);
    println!("---------------------------------------");
    println!(
        "完了: {}/{} が収束 | 平均 n_stable_regions = {:.2}",
        n_converged, cfg.runs, mean_regions
    );
    println!("設定   → {}/config.json", out_dir);
    println!("メトリクス → {}/metrics.csv", out_dir);
}

// ---------------------------------------------------------------------------
// sweep サブコマンド
// ---------------------------------------------------------------------------

fn cmd_sweep(args: SweepArgs) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let dir_name = format!("{}_sweep", timestamp);
    let sweep_dir = format!("{}/{}", args.output_dir, dir_name);
    fs::create_dir_all(&sweep_dir).expect("sweep ディレクトリの作成に失敗");

    let cfg = SweepConfig {
        width: args.width,
        height: args.height,
        features_min: args.features_min,
        features_max: args.features_max,
        features_step: args.features_step,
        traits_min: args.traits_min,
        traits_max: args.traits_max,
        traits_step: args.traits_step,
        runs: args.runs,
        max_events: args.max_events,
        seed: args.seed,
        output_dir: args.output_dir.clone(),
    };

    // 探索する (f, q) の列挙
    let mut feature_vals: Vec<usize> = Vec::new();
    {
        let mut f = cfg.features_min;
        while f <= cfg.features_max {
            feature_vals.push(f);
            f += cfg.features_step.max(1);
        }
    }
    let mut traits_vals: Vec<usize> = Vec::new();
    {
        let mut q = cfg.traits_min;
        while q <= cfg.traits_max {
            traits_vals.push(q);
            q += cfg.traits_step.max(1);
        }
    }

    let n_combos = feature_vals.len() * traits_vals.len();
    let n_total = n_combos * cfg.runs;

    println!("=== Axelrod 文化拡散モデル パラメータスイープ ===");
    println!(
        "グリッド: {}×{} | f={:?} | q={:?} | runs={} | max_events={}",
        cfg.width, cfg.height, feature_vals, traits_vals, cfg.runs, cfg.max_events
    );
    println!("シード (base): {:?}", cfg.seed);
    println!("合計 {} 試行 ({} 条件 × {} runs)", n_total, n_combos, cfg.runs);
    println!("出力先: {}", sweep_dir);
    println!("-----------------------------------------------");

    // sweep_config.json
    let sweep_config_json = serde_json::json!({
        "subcommand": "sweep",
        "width": cfg.width,
        "height": cfg.height,
        "features": {
            "min": cfg.features_min,
            "max": cfg.features_max,
            "step": cfg.features_step,
        },
        "traits": {
            "min": cfg.traits_min,
            "max": cfg.traits_max,
            "step": cfg.traits_step,
        },
        "runs": cfg.runs,
        "max_events": cfg.max_events,
        "seed": cfg.seed,
    });
    let config_path = format!("{}/sweep_config.json", sweep_dir);
    let file = File::create(&config_path).expect("sweep_config.json の作成に失敗");
    serde_json::to_writer_pretty(BufWriter::new(file), &sweep_config_json)
        .expect("sweep_config.json の書き込みに失敗");

    // sweep_summary.csv
    let summary_path = format!("{}/sweep_summary.csv", sweep_dir);
    let file = File::create(&summary_path).expect("sweep_summary.csv の作成に失敗");
    let mut wtr = Writer::from_writer(BufWriter::new(file));

    let mut idx = 0usize;
    for &features in &feature_vals {
        for &traits in &traits_vals {
            let mut sum_regions = 0.0f64;
            let mut n_converged = 0usize;

            for run in 0..cfg.runs {
                idx += 1;
                let seed = derive_seed(cfg.seed, features, traits, run);
                let (converged, n_events, m) = execute_run(
                    cfg.width,
                    cfg.height,
                    features,
                    traits,
                    cfg.max_events,
                    seed,
                );

                if converged {
                    n_converged += 1;
                }
                sum_regions += m.n_stable_regions as f64;

                let row = SweepRow {
                    features,
                    traits,
                    run,
                    width: cfg.width,
                    height: cfg.height,
                    seed,
                    converged,
                    n_events,
                    n_stable_regions: m.n_stable_regions,
                    max_region_size: m.max_region_size,
                    n_distinct_cultures: m.n_distinct_cultures,
                };
                wtr.serialize(row).expect("サマリ行の書き込みに失敗");
            }

            let mean = sum_regions / (cfg.runs as f64);
            println!(
                "[{}/{}] f={:<3} q={:<3} → converged={}/{} mean_regions={:.2}",
                idx, n_total, features, traits, n_converged, cfg.runs, mean
            );
        }
    }
    wtr.flush().expect("フラッシュに失敗");

    // latest シンボリックリンク
    update_latest_symlink(Path::new(&cfg.output_dir), &dir_name);

    println!("-----------------------------------------------");
    println!("スイープ完了．");
    println!("サマリ → {}/sweep_summary.csv", sweep_dir);
    println!("設定   → {}/sweep_config.json", sweep_dir);
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
