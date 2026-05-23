//! socsim フレームワーク上の Axelrod 文化拡散メカニズム．
//!
//! Axelrod (1997) のイベント駆動規則を socsim の [`Mechanism`] として実装する．
//! `Interaction` フェーズで発火し，1 エンジン step あたり `events_per_step` 回の
//! イベントを `ctx.world` 上で実行する(旧実装の `check_interval` 境界判定と論理的に
//! 等価)．イベントのサイト選択は内部で `ctx.rng` を使うため，スケジューラの
//! アクティベーション順序には依存しない．
//!
//! ステップ結果(実行イベント数・収束フラグ)は [`StepContext::scratch`] に書き込み，
//! ドライバが [`Simulation::scratch`](socsim_engine::Simulation::scratch) 経由で読む．
//! 収束(全隣接ペアで sim ∈ {0,1})を検知したら [`StepContext::request_stop`] で
//! エンジンに停止を要求する．

use socsim_core::{Mechanism, Phase, Result, SimRng, StepContext};

use rand::Rng;

use crate::world::AxelrodWorld;

/// 文化ベクトル間の類似度 (一致する特徴数 / 特徴数)．a.len() == b.len() を仮定．
#[inline]
pub fn similarity(a: &[usize], b: &[usize]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    if a.is_empty() {
        return 0.0;
    }
    let shared = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    shared as f64 / a.len() as f64
}

/// グローバル安定判定: 全ての隣接ペアについて sim ∈ {0, 1} が成り立つか．
/// 一つでも 0 < sim < 1 の隣接があれば false．
///
/// 事前計算済みの CSR 近傍表(`world.adjacency`)を使う．各ペアを2回見る(i→j と j→i)が
/// 判定は対称なので正しい(micro-opt より正当性を優先)．
pub fn is_stable(world: &AxelrodWorld) -> bool {
    for i in 0..world.n_sites() {
        for &j in world.adjacency.neighbors(i) {
            let sim = similarity(world.culture(i), world.culture(j));
            if sim > 0.0 && sim < 1.0 {
                return false;
            }
        }
    }
    true
}

/// 1イベントを実行する．戻り値は「相互作用(コピー)が起きたか」．
///
/// 手順:
///   1. アクティブサイト s を一様ランダムに選ぶ
///   2. s のフォン・ノイマン近傍から隣人 nb を一様ランダムに選ぶ
///   3. 類似度 sim = (一致特徴数 / f) を計算
///   4. sim == 0 または sim == 1 のときは相互作用しない
///   5. それ以外のときは確率 sim で，差がある特徴を1つランダムに選んで nb の値をコピー
fn run_event(world: &mut AxelrodWorld, rng: &mut SimRng) -> bool {
    let s = rng.gen_range(0..world.n_sites());
    let neighbors = world.adjacency.neighbors(s);
    if neighbors.is_empty() {
        return false;
    }
    let nb = neighbors[rng.gen_range(0..neighbors.len())];

    // 同じインデックスが来ることは構造上ないが念のため．
    if s == nb {
        return false;
    }

    let sim = similarity(world.culture(s), world.culture(nb));

    // sim == 0: 共通点が無いので相互作用しない．
    // sim == 1: 全て同じなので模倣する余地がない．
    if sim <= 0.0 || sim >= 1.0 {
        return false;
    }

    // 確率 sim で相互作用．
    if !rng.gen_bool(sim) {
        return false;
    }

    // 差がある特徴を列挙して1つランダムに選ぶ．
    let diffs: Vec<usize> = (0..world.n_features)
        .filter(|&f| world.culture(s)[f] != world.culture(nb)[f])
        .collect();

    // sim < 1 なので diffs は必ず 1 以上．
    debug_assert!(!diffs.is_empty());
    let feat = diffs[rng.gen_range(0..diffs.len())];

    let new_val = world.culture(nb)[feat];
    world
        .cells
        .get_idx_mut(s)
        .expect("idx が範囲外(run_event write)")[feat] = new_val;
    true
}

/// 1 step あたり `events_per_step` 回のイベントをバッチ実行するメカニズム．
pub struct AxelrodInteractionMechanism {
    /// 1 エンジン step あたりのイベント数(既定 = n_sites)．
    pub events_per_step: usize,
}

impl Mechanism<AxelrodWorld> for AxelrodInteractionMechanism {
    fn name(&self) -> &str {
        "axelrod_interaction"
    }

    fn phases(&self) -> &'static [Phase] {
        &[Phase::Interaction]
    }

    fn apply(&mut self, _phase: Phase, ctx: &mut StepContext<'_, AxelrodWorld>) -> Result<()> {
        let mut events_run = 0usize;
        for _ in 0..self.events_per_step {
            run_event(ctx.world, ctx.rng);
            events_run += 1;
        }

        let stable = is_stable(ctx.world);

        // ステップ結果を scratch に書き出す(ドライバが読む)．
        ctx.scratch.insert("delta_events", events_run);
        ctx.scratch.insert("converged", stable);

        if stable {
            ctx.request_stop();
        }

        Ok(())
    }
}
