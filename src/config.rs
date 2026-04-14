/// `simulate` サブコマンドの設定
#[derive(Debug, Clone)]
pub struct SimulateConfig {
    /// グリッド幅
    pub width: usize,
    /// グリッド高さ
    pub height: usize,
    /// 特徴数 f (文化ベクトルの長さ)
    pub features: usize,
    /// 特性数 q (各特徴が取りうる値の数)
    pub traits: usize,
    /// 試行回数
    pub runs: usize,
    /// 1試行あたりの最大イベント数（収束しない場合の上限）
    pub max_events: usize,
    /// 乱数シードのベース値（Noneならランダム）
    pub seed: Option<u64>,
    /// 出力ディレクトリのベース
    pub output_dir: String,
}

/// `sweep` サブコマンドの設定
#[derive(Debug, Clone)]
pub struct SweepConfig {
    pub width: usize,
    pub height: usize,
    pub features_min: usize,
    pub features_max: usize,
    pub features_step: usize,
    pub traits_min: usize,
    pub traits_max: usize,
    pub traits_step: usize,
    pub runs: usize,
    pub max_events: usize,
    pub seed: Option<u64>,
    pub output_dir: String,
}

/// Table 7-2 に近いデフォルト値（10×10グリッド）
impl Default for SimulateConfig {
    fn default() -> Self {
        SimulateConfig {
            width: 10,
            height: 10,
            features: 5,
            traits: 10,
            runs: 10,
            max_events: 1_000_000,
            seed: Some(42),
            output_dir: "results".to_string(),
        }
    }
}
