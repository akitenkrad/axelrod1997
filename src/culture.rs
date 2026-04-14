use rand::Rng;

use crate::grid::Grid;

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

/// 1イベントを実行する．
/// 戻り値は「相互作用（コピー）が起きたか」．
///
/// 手順:
///   1. アクティブサイト s を一様ランダムに選ぶ
///   2. s のフォン・ノイマン近傍から隣人 nb を一様ランダムに選ぶ
///   3. 類似度 sim = (一致特徴数 / f) を計算
///   4. sim == 0 または sim == 1 のときは相互作用しない
///   5. それ以外のときは確率 sim で，差がある特徴を1つランダムに選んで nb の値をコピー
pub fn step(grid: &mut Grid, rng: &mut impl Rng) -> bool {
    let s = rng.gen_range(0..grid.n_sites());
    let neighbors = grid.neighbors_4(s);
    if neighbors.is_empty() {
        return false;
    }
    let nb = neighbors[rng.gen_range(0..neighbors.len())];

    // 同じインデックスが来ることは構造上ないが念のため
    if s == nb {
        return false;
    }

    let sim = similarity(&grid.sites[s], &grid.sites[nb]);

    // sim == 0: 共通点が無いので相互作用しない
    // sim == 1: 全て同じなので模倣する余地がない
    if sim <= 0.0 || sim >= 1.0 {
        return false;
    }

    // 確率 sim で相互作用
    if !rng.gen_bool(sim) {
        return false;
    }

    // 差がある特徴を列挙して1つランダムに選ぶ
    let diffs: Vec<usize> = (0..grid.n_features)
        .filter(|&f| grid.sites[s][f] != grid.sites[nb][f])
        .collect();

    // sim < 1 なので diffs は必ず 1 以上
    debug_assert!(!diffs.is_empty());
    let feat = diffs[rng.gen_range(0..diffs.len())];

    let new_val = grid.sites[nb][feat];
    grid.sites[s][feat] = new_val;
    true
}

/// グローバル安定判定: 全ての隣接ペアについて sim ∈ {0, 1} が成り立つか．
/// 一つでも 0 < sim < 1 の隣接があれば false．
pub fn is_stable(grid: &Grid) -> bool {
    let n = grid.n_sites();
    for i in 0..n {
        let x = i % grid.width;
        let y = i / grid.width;
        // 右と下の隣接のみ見れば全ペアを1回ずつカバーできる
        if x + 1 < grid.width {
            let j = i + 1;
            let sim = similarity(&grid.sites[i], &grid.sites[j]);
            if sim > 0.0 && sim < 1.0 {
                return false;
            }
        }
        if y + 1 < grid.height {
            let j = i + grid.width;
            let sim = similarity(&grid.sites[i], &grid.sites[j]);
            if sim > 0.0 && sim < 1.0 {
                return false;
            }
        }
    }
    true
}

/// 単一試行の結果
#[derive(Debug, Clone)]
pub struct RunResult {
    pub converged: bool,
    pub n_events: usize,
    pub grid: Grid,
}

/// 安定するか `max_events` に達するまでイベントを繰り返す．
/// 安定判定は全サイト数 n = width*height イベントごとに行う（毎イベント判定は高コスト）．
pub fn run_until_stable(
    mut grid: Grid,
    max_events: usize,
    rng: &mut impl Rng,
) -> RunResult {
    let check_interval = grid.n_sites().max(1);
    let mut n_events = 0usize;
    let mut converged = false;

    while n_events < max_events {
        step(&mut grid, rng);
        n_events += 1;

        if n_events % check_interval == 0 && is_stable(&grid) {
            converged = true;
            break;
        }
    }

    // ループ終了後にも安定チェック（max_events に到達した場合でも念のため）
    if !converged && is_stable(&grid) {
        converged = true;
    }

    RunResult {
        converged,
        n_events,
        grid,
    }
}
