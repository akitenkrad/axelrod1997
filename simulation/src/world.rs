//! socsim フレームワーク上の Axelrod 文化拡散モデルの世界状態．
//!
//! `AxelrodWorld` は socsim の [`WorldState`] を実装する．各サイトは文化ベクトル
//! (`Vec<usize>`) を持ち，盤面は全セル占有・エージェント移動なしで，文化ベクトルが
//! mutate するだけである．したがって占有インデックス([`socsim_grid::GridIndex`])は
//! 使わず，[`socsim_grid::Grid`] は近傍/位相の計算にのみ用いる．
//!
//! セル状態は socsim-grid の [`CellGrid<Culture>`] に保持する(セル値 = 文化ベクトル，
//! フラットインデックス `idx = r*cols + c`)．近傍は同じグリッドから
//! [`Grid::adjacency`](socsim_grid::Grid::adjacency) で CSR の [`Adjacency`] を一度だけ
//! 事前計算して保持する(ホットループ高速化)．`Adjacency::neighbors(idx)` は旧実装の
//! 手書き近傍表と同じ順序(ソート済み row-major のフラットインデックス)を返すため，
//! RNG を消費する選択の draw 順序は変わらない．

use socsim_core::{AgentId, SimClock, SimRng, WorldState};
use socsim_grid::{Adjacency, Boundary, CellGrid, Grid, Neighborhood};

use rand::Rng;

/// 1サイトの文化ベクトル．`culture[i]` は i 番目の特徴の特性値 (0..n_traits)．
pub type Culture = Vec<usize>;

/// Axelrod 文化拡散モデルの世界状態．
#[derive(Clone)]
pub struct AxelrodWorld {
    /// シミュレーションクロック
    pub clock: SimClock,
    /// 各サイトの文化ベクトルを保持するセルグリッド(セル値 = 文化ベクトル，
    /// フラットインデックス idx = r*cols + c)．セル状態の単一の真実源．
    pub cells: CellGrid<Culture>,
    /// 特徴数 f (文化ベクトルの長さ)
    pub n_features: usize,
    /// 特性数 q (各特徴が取りうる値の数)．init 後は保持のみ．
    #[allow(dead_code)]
    pub n_traits: usize,
    /// 各サイトのフォン・ノイマン近傍(フラットインデックス)の CSR 事前計算表．
    /// `cells.grid()` から構築する．
    pub adjacency: Adjacency,
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
        debug_assert_eq!(cultures.len(), width * height);
        let grid = Grid::new(height, width, Boundary::Fixed);
        // VonNeumann 近傍を CSR で事前計算(フラット idx → 近傍フラット idx)．
        let adjacency = grid.adjacency(Neighborhood::VonNeumann);
        // 文化ベクトルを CellGrid に格納(row-major で消費，順序は不変)．
        let mut iter = cultures.into_iter();
        let cells = CellGrid::from_fn(grid, |_, _| {
            iter.next().expect("cultures の要素数が width*height に満たない")
        });
        AxelrodWorld {
            clock: SimClock::new(t_max),
            cells,
            n_features,
            n_traits,
            adjacency,
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

    /// フラットインデックス `idx` の文化ベクトルを借用する．
    #[inline]
    pub fn culture(&self, idx: usize) -> &Culture {
        self.cells
            .get_idx(idx)
            .expect("idx が範囲外(culture)")
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
