//! runvault への記録の共通部分．
//!
//! 論文メタデータ (research) は `simulate` / `sweep` のどちらでも同一なので，
//! ここ 1 箇所で組み立てる．試行 (trial) 1 本の終端行と，run 全体の集約指標も
//! ここに集める．

use runvault::{Replication, Run, Target, Work};
use serde::Serialize;

use crate::metrics::RunMetrics;

/// runvault 上の実験名．`runvault path --experiment` に渡す値でもある．
pub const EXPERIMENT: &str = "axelrod";
/// リポジトリの安定 id．git remote の名前とは独立に固定する．
pub const REPO_ID: &str = "axelrod1997";
/// 分野．`simulation` を名乗ると `master_seed` が必須になる．
pub const DOMAIN: &str = "simulation";

/// 試行の時間軸の単位．
///
/// Axelrod のモデルの時間の刻みは 1 イベント (1 サイトの活性化) である．エンジンが
/// `n_sites` イベントをまとめて 1 step として回しているのは安定判定を安く済ませる
/// ための実装都合にすぎず，モデルの時間ではない．記録するのはモデル側の時間なので
/// `t = n_events`，`budget = max_events` を runvault の語彙 `event` として書く．
const T_UNIT: &str = "event";

/// この再現実験が対象としている論文．
///
/// vault の書誌表に DOI が無い (欄が「—」) ので，同定は vault の `paper-id` で行う．
/// `sweep` が再現するのは f×q の非対称効果だが，どの表・図を掴むかは
/// `--features-*` / `--traits-*` 次第で run ごとに変わるため，
/// 表・図の `Target` はここでは付けない (claim だけを共通の対象として持つ)．
pub fn replication() -> Replication {
    Work::paper_id("P00001493")
        .title("The Dissemination of Culture: A Model with Local Convergence and Global Polarization")
        .year(1997)
        .source_version("published")
        .target(Target::claim(
            "feature-trait-asymmetry",
            "More features reduce the number of stable regions while more traits increase it",
        ))
        .obsidian_note("研究/98_論文レポート/80-再現実験/実装完了/axelrod1997/設計書.md")
}

/// 試行 1 本の結果．`events.jsonl` の 1 行と，run 全体の集約の材料になる．
#[derive(Debug, Clone)]
pub struct Trial {
    /// 0 始まりの試行番号．`unit_id` はこれから作る．
    pub index: usize,
    /// この試行に実際に使われた派生シード．
    pub seed: u64,
    /// 安定 (収束) に達したか．
    pub converged: bool,
    /// 実行したイベント数 (`max_events` で頭打ち)．
    pub n_events: usize,
    /// 1 試行あたりのイベント数の上限．
    pub max_events: usize,
    /// 最終盤面の集計．
    pub metrics: RunMetrics,
}

/// `events.jsonl` に書く終端行．
///
/// 先頭 6 フィールドは runvault の予約語 (`terminal` はこれを全部要求する)．
/// 残りは試行ごとの自由欄で，`sweep` の子 run でも同じ形になる．
#[derive(Serialize)]
struct TerminalEvent {
    unit_id: String,
    t: u64,
    t_unit: &'static str,
    outcome: &'static str,
    censored: bool,
    budget: u64,
    seed: u64,
    n_stable_regions: usize,
    max_region_size: usize,
    n_distinct_cultures: usize,
}

/// 試行 1 本を `terminal` イベントとして書く．
///
/// 打ち切り (`censored`) の行は `t == budget` でなければならない．ドライバは
/// `t_max = ceil(max_events / events_per_step)` step 回して `n_events` を
/// `max_events` で頭打ちにするので，収束しなかった試行は必ず上限に達している．
/// この不変条件は runvault が `log_event` の書き込み時に検査するので，ここでは
/// 二重に持たない — 前提が崩れれば矛盾した行はファイルに届かず，下の
/// `unwrap_or_else` が試行番号つきで落ちる．
pub fn log_trial(run: &mut Run, trial: &Trial) {
    let censored = !trial.converged;

    let event = TerminalEvent {
        unit_id: format!("trial-{}", trial.index),
        t: trial.n_events as u64,
        t_unit: T_UNIT,
        outcome: if trial.converged {
            "converged"
        } else {
            "unconverged"
        },
        censored,
        budget: trial.max_events as u64,
        seed: trial.seed,
        n_stable_regions: trial.metrics.n_stable_regions,
        max_region_size: trial.metrics.max_region_size,
        n_distinct_cultures: trial.metrics.n_distinct_cultures,
    };
    run.log_event("terminal", &event)
        .unwrap_or_else(|e| panic!("試行 {} の terminal イベントの記録に失敗: {e}", trial.index));
}

/// run 全体を 1 つの値で表す指標．
///
/// 試行ごとの値は `events.jsonl` の担当なので，ここには集約しか書かない．
/// 試行ごとの `converged` を指標にすると (`run_uid`, `step`, `scope`, `name`) が
/// 重複するので，収束は率にして 1 行にする．
pub fn log_run_summary(run: &mut Run, trials: &[Trial]) {
    let n = trials.len();
    assert!(n > 0, "試行が 1 本もありません");
    let n_f = n as f64;

    let n_converged = trials.iter().filter(|t| t.converged).count();
    let mean = |f: &dyn Fn(&Trial) -> f64| trials.iter().map(f).sum::<f64>() / n_f;

    run.log_metrics(
        "run",
        &[
            ("n_units", n_f),
            ("n_converged", n_converged as f64),
            ("convergence_rate", n_converged as f64 / n_f),
            ("mean_n_stable_regions", mean(&|t| t.metrics.n_stable_regions as f64)),
            ("mean_max_region_size", mean(&|t| t.metrics.max_region_size as f64)),
            ("mean_n_distinct_cultures", mean(&|t| t.metrics.n_distinct_cultures as f64)),
            ("mean_n_events", mean(&|t| t.n_events as f64)),
        ],
    )
    .expect("run スコープの指標の記録に失敗");
}

// ---------------------------------------------------------------------------
// シードの派生
// ---------------------------------------------------------------------------

/// 試行 1 本のシードを base seed から決定的に派生させる．
///
/// `master_seed` として記録するのは `base` の方で，実際に各試行が使うシードは
/// これで作る．`(base, features, traits, index)` が同じなら常に同じ値を返し，
/// どれか 1 つでも違えば別の値になる — この性質が壊れると，記録した
/// `master_seed` から run を組み直せなくなる．
pub fn trial_seed(base: u64, features: usize, traits: usize, index: usize) -> u64 {
    socsim_core::derive_seed(base, &[features as u64, traits as u64, index as u64])
}

#[cfg(test)]
mod tests {
    use super::trial_seed;

    #[test]
    fn same_inputs_give_the_same_seed() {
        assert_eq!(trial_seed(42, 5, 10, 3), trial_seed(42, 5, 10, 3));
        for index in 0..8 {
            assert_eq!(
                trial_seed(2026, 15, 5, index),
                trial_seed(2026, 15, 5, index),
                "index={index} で再現しなかった"
            );
        }
    }

    #[test]
    fn a_different_base_gives_a_different_seed() {
        assert_ne!(trial_seed(42, 5, 10, 0), trial_seed(43, 5, 10, 0));
        // base だけを動かして，試行番号をまたいで衝突しないことも見る．
        for index in 0..8 {
            assert_ne!(
                trial_seed(42, 5, 10, index),
                trial_seed(43, 5, 10, index),
                "index={index} で base の違いが消えた"
            );
        }
    }

    #[test]
    fn each_coordinate_changes_the_seed() {
        let base = trial_seed(42, 5, 10, 0);
        assert_ne!(base, trial_seed(42, 6, 10, 0), "features が効いていない");
        assert_ne!(base, trial_seed(42, 5, 11, 0), "traits が効いていない");
        assert_ne!(base, trial_seed(42, 5, 10, 1), "index が効いていない");
    }

    #[test]
    fn one_condition_gives_distinct_seeds_across_trials() {
        let seeds: std::collections::BTreeSet<u64> =
            (0..64).map(|i| trial_seed(42, 5, 10, i)).collect();
        assert_eq!(seeds.len(), 64, "同一条件の試行でシードが衝突した");
    }

    /// 具体値を固定する．
    ///
    /// ここが変わるのは socsim の `derive_seed` が変わったときで，そのときは
    /// 過去の run と結果を比較できなくなっている．Cargo.lock が socsim の commit を
    /// 固定しているので，この値は依存を上げたときにだけ動く．
    #[test]
    fn golden_values_are_pinned() {
        assert_eq!(trial_seed(42, 5, 10, 0), 8_371_972_187_346_912_400);
        assert_eq!(trial_seed(42, 5, 10, 1), 8_371_973_286_858_540_611);
        assert_eq!(trial_seed(42, 15, 15, 9), 2_131_266_233_058_076_096);
    }
}
