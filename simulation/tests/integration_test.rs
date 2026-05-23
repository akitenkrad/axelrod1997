//! Axelrod (1997) 文化拡散モデルの統合テスト．
//!
//! `axelrod_culture` ライブラリクレートの公開 API に対して，
//! ・`AxelrodWorld::random_init` の形状と値域
//! ・`similarity` の境界値
//! ・`is_stable` の一様／非一様グリッドでの振る舞い
//! ・`count_stable_regions` の極端ケース
//! ・小規模グリッドでのエンドツーエンド実行
//! を検証する．

use socsim_core::SimRng;

use axelrod_culture::mechanisms::{is_stable, similarity};
use axelrod_culture::metrics::count_stable_regions;
use axelrod_culture::simulation::run;
use axelrod_culture::world::AxelrodWorld;

// 統合テストで使う十分大きな t_max(安定判定・地域カウントは t_max に依存しない)．
const TEST_T_MAX: u64 = 1_000;

// --------------------------------------------------------------------------- //
// AxelrodWorld::random_init — 形状と値域
// --------------------------------------------------------------------------- //

#[test]
fn random_init_has_correct_shape_and_value_range() {
    let width = 5;
    let height = 4;
    let features = 6;
    let traits = 7;
    let mut rng = SimRng::from_seed(123);

    let world = AxelrodWorld::random_init(width, height, features, traits, &mut rng, TEST_T_MAX);

    // 形状
    assert_eq!(world.width(), width);
    assert_eq!(world.height(), height);
    assert_eq!(world.n_features, features);
    assert_eq!(world.cultures.len(), width * height);

    // 各サイトの文化ベクトルは長さ f
    for site in &world.cultures {
        assert_eq!(site.len(), features);
        // 各特性値は 0..q の範囲に収まる
        for &v in site {
            assert!(v < traits, "特性値 {} が n_traits={} を超えた", v, traits);
        }
    }
}

#[test]
fn random_init_is_deterministic_with_same_seed() {
    let mut rng1 = SimRng::from_seed(42);
    let mut rng2 = SimRng::from_seed(42);
    let g1 = AxelrodWorld::random_init(4, 4, 3, 5, &mut rng1, TEST_T_MAX);
    let g2 = AxelrodWorld::random_init(4, 4, 3, 5, &mut rng2, TEST_T_MAX);
    assert_eq!(g1.cultures, g2.cultures);
}

// --------------------------------------------------------------------------- //
// similarity — 同一・完全不一致・部分一致
// --------------------------------------------------------------------------- //

#[test]
fn similarity_identical_vectors_is_one() {
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![1, 2, 3, 4, 5];
    assert_eq!(similarity(&a, &b), 1.0);
}

#[test]
fn similarity_disjoint_vectors_is_zero() {
    let a = vec![0, 0, 0, 0];
    let b = vec![1, 1, 1, 1];
    assert_eq!(similarity(&a, &b), 0.0);
}

#[test]
fn similarity_partial_match() {
    let a = vec![1, 2, 3, 4];
    let b = vec![1, 2, 9, 9]; // 2 / 4 一致
    assert!((similarity(&a, &b) - 0.5).abs() < 1e-12);

    let c = vec![1, 2, 3, 9]; // 3 / 4 一致
    assert!((similarity(&a, &c) - 0.75).abs() < 1e-12);
}

// --------------------------------------------------------------------------- //
// is_stable — 一様／ランダム
// --------------------------------------------------------------------------- //

#[test]
fn is_stable_uniform_grid_is_stable() {
    // 全サイトが同じ文化ベクトル → どの隣接ペアでも sim = 1
    let width = 5;
    let height = 5;
    let features = 4;
    let culture = vec![0, 1, 2, 3];
    let world = AxelrodWorld::new(
        width,
        height,
        features,
        4,
        vec![culture; width * height],
        TEST_T_MAX,
    );
    assert!(is_stable(&world));
}

#[test]
fn is_stable_random_grid_is_generally_not_stable() {
    // f=5, q=10, 10×10 のランダム初期化は，ほぼ確実にどこかに 0 < sim < 1 の隣接ペアを含む
    let mut rng = SimRng::from_seed(7);
    let world = AxelrodWorld::random_init(10, 10, 5, 10, &mut rng, TEST_T_MAX);
    assert!(
        !is_stable(&world),
        "ランダムな 10×10 (f=5, q=10) グリッドが即座に安定しているのは極めて稀"
    );
}

// --------------------------------------------------------------------------- //
// count_stable_regions — 極端ケース
// --------------------------------------------------------------------------- //

#[test]
fn count_stable_regions_uniform_grid_is_one() {
    let width = 4;
    let height = 4;
    let culture = vec![2, 2, 2];
    let world = AxelrodWorld::new(
        width,
        height,
        3,
        3,
        vec![culture; width * height],
        TEST_T_MAX,
    );
    let m = count_stable_regions(&world);
    assert_eq!(m.n_stable_regions, 1);
    assert_eq!(m.max_region_size, width * height);
    assert_eq!(m.n_distinct_cultures, 1);
}

#[test]
fn count_stable_regions_all_distinct_is_n() {
    // 各サイトに異なる文化ベクトルを与えると，連結成分数はサイト数と一致する
    let width = 3;
    let height = 3;
    let features = 2;
    let n = width * height;
    // 各サイトに異なるベクトルを作る．f=2, q=10 なら 9 通りは容易
    let cultures: Vec<Vec<usize>> = (0..n).map(|i| vec![i, i + 1]).collect();
    let world = AxelrodWorld::new(width, height, features, 100, cultures, TEST_T_MAX);
    let m = count_stable_regions(&world);
    assert_eq!(m.n_stable_regions, n);
    assert_eq!(m.max_region_size, 1);
    assert_eq!(m.n_distinct_cultures, n);
}

// --------------------------------------------------------------------------- //
// エンドツーエンド: 小規模グリッドでの収束
// --------------------------------------------------------------------------- //

#[test]
fn small_end_to_end_converges_with_seed() {
    // 4×4, f=2, q=2 は非常に少ない状態空間なので，高い確率で短時間に収束する
    let result = run(4, 4, 2, 2, 100_000, 2026);

    // 小さい状態空間なので max_events に到達せず収束する
    assert!(result.converged, "4×4 (f=2, q=2) で収束しなかった");

    // 収束後は is_stable が真
    assert!(is_stable(&result.world));

    // 安定地域数は 1..=16 の範囲に収まる
    let m = count_stable_regions(&result.world);
    assert!(m.n_stable_regions >= 1);
    assert!(m.n_stable_regions <= 16);
    assert!(m.max_region_size >= 1);
    assert!(m.max_region_size <= 16);
}
