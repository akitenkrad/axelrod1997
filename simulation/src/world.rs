//! socsim フレームワーク上の Axelrod 文化拡散モデルの世界状態．
//!
//! `AxelrodWorld` は socsim の [`WorldState`] を実装する．各サイトは文化ベクトル
//! (`Vec<usize>`) を持ち，盤面は全セル占有・エージェント移動なしで，文化ベクトルが
//! mutate するだけである．したがって占有インデックス([`socsim_grid::GridIndex`])は
//! 使わず，[`socsim_grid::Grid`] は近傍/位相の計算にのみ用いる．
//!
//! 旧実装(`grid.rs`)の手書き近傍計算(`neighbors_4`)は socsim-grid の
//! [`Grid::neighbors`] でフォン・ノイマン近傍を初期化時に事前計算し，フラット
//! インデックス(`r*width + c`)の隣接表 `neighbors` として保持する(ホットループ高速化)．

use socsim_core::{AgentId, SimClock, SimRng, WorldState};
use socsim_grid::{Boundary, Grid, Neighborhood};

use rand::Rng;

/// 1サイトの文化ベクトル．`culture[i]` は i 番目の特徴の特性値 (0..n_traits)．
pub type Culture = Vec<usize>;

/// Axelrod 文化拡散モデルの世界状態．
#[derive(Clone)]
pub struct AxelrodWorld {
    /// シミュレーションクロック
    pub clock: SimClock,
    /// 近傍/位相計算用のグリッド(rows=height, cols=width)．近傍事前計算後は保持のみ．
    #[allow(dead_code)]
    pub grid: Grid,
    /// 各サイトの文化ベクトル．フラットインデックス(idx = r*width + c)．
    pub cultures: Vec<Culture>,
    /// 特徴数 f (文化ベクトルの長さ)
    pub n_features: usize,
    /// 特性数 q (各特徴が取りうる値の数)．init 後は保持のみ．
    #[allow(dead_code)]
    pub n_traits: usize,
    /// 各サイトのフォン・ノイマン近傍(フラットインデックス)の事前計算表．
    pub neighbors: Vec<Vec<usize>>,
    /// グリッド幅(列数)．
    width: usize,
    /// グリッド高さ(行数)．
    height: usize,
}

impl AxelrodWorld {
    /// グリッド寸法と文化ベクトル列から世界状態を構築する(近傍を事前計算)．
    pub fn new(
        width: usize,
        height: usize,
        n_features: usize,
        n_traits: usize,
        cultures: Vec<Culture>,
        t_max: u64,
    ) -> Self {
        let grid = Grid::new(height, width, Boundary::Fixed);
        let n = width * height;
        // 近傍を事前計算: フラット idx → (r, c) → VonNeumann 近傍 → フラット idx．
        let mut neighbors: Vec<Vec<usize>> = Vec::with_capacity(n);
        for idx in 0..n {
            let r = idx / width;
            let c = idx % width;
            let ns: Vec<usize> = grid
                .neighbors(r, c, Neighborhood::VonNeumann)
                .into_iter()
                .map(|(nr, nc)| nr * width + nc)
                .collect();
            neighbors.push(ns);
        }
        AxelrodWorld {
            clock: SimClock::new(t_max),
            grid,
            cultures,
            n_features,
            n_traits,
            neighbors,
            width,
            height,
        }
    }

    /// ランダム初期化: 各サイトに (0..q) から独立に f 個の特性値を割り当てる．
    pub fn random_init(
        width: usize,
        height: usize,
        n_features: usize,
        n_traits: usize,
        rng: &mut SimRng,
        t_max: u64,
    ) -> Self {
        let n = width * height;
        let cultures: Vec<Culture> = (0..n)
            .map(|_| (0..n_features).map(|_| rng.gen_range(0..n_traits)).collect())
            .collect();
        Self::new(width, height, n_features, n_traits, cultures, t_max)
    }

    /// サイト数(= width * height)．
    #[inline]
    pub fn n_sites(&self) -> usize {
        self.width * self.height
    }

    /// グリッド幅(列数)．
    #[inline]
    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width
    }

    /// グリッド高さ(行数)．
    #[inline]
    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.height
    }
}

impl WorldState for AxelrodWorld {
    fn agent_ids(&self) -> Vec<AgentId> {
        (0..self.n_sites()).map(|i| AgentId(i as u64)).collect()
    }

    fn clock(&self) -> &SimClock {
        &self.clock
    }

    fn clock_mut(&mut self) -> &mut SimClock {
        &mut self.clock
    }
}
