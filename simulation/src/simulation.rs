//! socsim フレームワーク上の Axelrod モデル実行ドライバ．
//!
//! 旧実装の `run_until_stable` を socsim エンジン上に載せ替えたもの．
//! 1 エンジン step = `events_per_step`(= n_sites)回のイベントというバッチ方式で，
//! 旧実装が `check_interval`(= n_sites)境界でしか安定判定しなかった挙動と論理的に
//! 等価である．`t_max = ceil(max_events / events_per_step)`．
//!
//! 早期停止は [`AxelrodInteractionMechanism`] が安定検知時に
//! [`StepContext::request_stop`](socsim_core::StepContext::request_stop) で要求し，
//! ドライバは [`Simulation::stop_requested`](socsim_engine::Simulation::stop_requested)
//! を見てループを抜ける．ステップ結果(実行イベント数・収束)は
//! [`Simulation::scratch`](socsim_engine::Simulation::scratch) 経由で受け取る．

use socsim_core::{derive_seed, SimRng};
use socsim_engine::{SequentialScheduler, SimulationBuilder};

use crate::mechanisms::AxelrodInteractionMechanism;
use crate::world::AxelrodWorld;

// 単一の root シードから用途別の独立な決定論的 RNG ストリームを派生させるラベル．
/// 世界初期化(文化ベクトルのランダム割り当て)用 RNG のラベル．
const RNG_WORLD_INIT: u64 = 0;
/// socsim エンジン(メカニズム内のイベント RNG)用 RNG のラベル．
const RNG_ENGINE: u64 = 1;

/// 単一試行の結果．
pub struct RunResult {
    /// 安定(収束)に達したか．
    pub converged: bool,
    /// 実行したイベント数(max_events で頭打ち)．
    pub n_events: usize,
    /// 最終盤面．
    pub world: AxelrodWorld,
}

/// 単一試行を実行する．
///
/// `events_per_step = n_sites = width*height`(>=1)．`t_max = ceil(max_events / events_per_step)`．
/// 初期化 RNG とエンジン RNG は単一 `seed` から別ラベルで派生させる．
pub fn run(
    width: usize,
    height: usize,
    features: usize,
    traits: usize,
    max_events: usize,
    seed: u64,
) -> RunResult {
    let events_per_step = (width * height).max(1);
    let t_max = (max_events.div_ceil(events_per_step)) as u64;

    // 世界初期化(文化ベクトルのランダム割り当て用 RNG．seed から派生)．
    let mut init_rng = SimRng::from_seed(derive_seed(seed, &[RNG_WORLD_INIT]));
    let world = AxelrodWorld::random_init(width, height, features, traits, &mut init_rng, t_max);

    // エンジンを構築(エンジン RNG も seed から別ラベルで派生)．
    let mut sim = SimulationBuilder::new(world)
        .scheduler(Box::new(SequentialScheduler))
        .seed(derive_seed(seed, &[RNG_ENGINE]))
        .add_mechanism(Box::new(AxelrodInteractionMechanism { events_per_step }))
        .build();

    let mut n_events = 0usize;
    let mut converged = false;

    // run_observed は `while !clock.is_done() && !stop_requested` で step を回し，
    // 実行された各 step(停止要求 step を含む)につき observer を1回呼ぶ．これは旧来の
    // 手書きループ(`for _ in 0..t_max { step(); read scratch; if stop break }`)と
    // step 数・RNG 使用が等価である．
    sim.run_observed(|report| {
        let delta = *report
            .scratch
            .get::<usize>("delta_events")
            .expect("delta_events が scratch に存在しません");
        n_events += delta;

        converged = *report
            .scratch
            .get::<bool>("converged")
            .expect("converged が scratch に存在しません");
    })
    .expect("シミュレーションの実行に失敗");

    // 実行イベント数を max_events で頭打ち(バッチ境界で超過しうるため)．
    if n_events > max_events {
        n_events = max_events;
    }

    RunResult {
        converged,
        n_events,
        world: sim.world().clone(),
    }
}
