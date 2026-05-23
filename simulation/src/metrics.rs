use std::collections::{HashSet, VecDeque};

use crate::world::AxelrodWorld;

/// 1試行分の集計メトリクス
#[derive(Debug, Clone)]
pub struct RunMetrics {
    /// 安定文化地域数（同一文化の4連結成分の数）．Table 7-2 の主要指標
    pub n_stable_regions: usize,
    /// 最大地域のサイズ（サイト数）
    pub max_region_size: usize,
    /// 盤面上に現れる相異なる文化ベクトルの数
    pub n_distinct_cultures: usize,
}

/// 安定文化地域数をカウントする．
/// 「同じ文化ベクトルを持つサイトの4連結成分」を BFS で列挙する．
/// 連結は事前計算済みのフォン・ノイマン近傍表(`world.adjacency`)を使う．
pub fn count_stable_regions(world: &AxelrodWorld) -> RunMetrics {
    let n = world.n_sites();
    let mut visited = vec![false; n];
    let mut n_regions = 0usize;
    let mut max_region_size = 0usize;

    for start in 0..n {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        n_regions += 1;

        let mut size = 1usize;
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);

        while let Some(cur) = queue.pop_front() {
            for &nb in world.adjacency.neighbors(cur) {
                if !visited[nb] && world.culture(cur) == world.culture(nb) {
                    visited[nb] = true;
                    size += 1;
                    queue.push_back(nb);
                }
            }
        }

        if size > max_region_size {
            max_region_size = size;
        }
    }

    // 相異なる文化ベクトルの数は HashSet で数える．
    let mut set: HashSet<&Vec<usize>> = HashSet::new();
    for site in world.cells.cells() {
        set.insert(site);
    }

    RunMetrics {
        n_stable_regions: n_regions,
        max_region_size,
        n_distinct_cultures: set.len(),
    }
}
