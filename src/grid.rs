use rand::Rng;

/// 1サイトの文化ベクトル．`culture[i]` は i 番目の特徴の特性値 (0..n_traits)
pub type Culture = Vec<usize>;

/// 非トーラスの矩形グリッド
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub n_features: usize,
    #[allow(dead_code)]
    pub n_traits: usize,
    /// サイトの1次元配列（row-major: idx = y * width + x）
    pub sites: Vec<Culture>,
}

impl Grid {
    /// ランダム初期化: 各サイトに (0..q) から独立に f 個の特性値を割り当てる
    pub fn random_init(
        width: usize,
        height: usize,
        n_features: usize,
        n_traits: usize,
        rng: &mut impl Rng,
    ) -> Self {
        let n = width * height;
        let sites: Vec<Culture> = (0..n)
            .map(|_| (0..n_features).map(|_| rng.gen_range(0..n_traits)).collect())
            .collect();
        Grid {
            width,
            height,
            n_features,
            n_traits,
            sites,
        }
    }

    #[inline]
    pub fn n_sites(&self) -> usize {
        self.width * self.height
    }

    /// フォン・ノイマン近傍（上下左右の4サイト，非トーラス）．
    /// 端のサイトでは近傍数が 2 または 3 になる．
    pub fn neighbors_4(&self, idx: usize) -> Vec<usize> {
        let x = idx % self.width;
        let y = idx / self.width;
        let mut ns = Vec::with_capacity(4);
        if x > 0 {
            ns.push(idx - 1);
        }
        if x + 1 < self.width {
            ns.push(idx + 1);
        }
        if y > 0 {
            ns.push(idx - self.width);
        }
        if y + 1 < self.height {
            ns.push(idx + self.width);
        }
        ns
    }
}
