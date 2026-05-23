//! Axelrod (1997) 文化拡散モデルの再現実装ライブラリ．
//!
//! socsim フレームワーク上に構築した文化拡散モデルの公開 API を提供する．
//! 世界状態(`world`)・相互作用メカニズム(`mechanisms`)・実行ドライバ
//! (`simulation`)・集計メトリクス(`metrics`)・設定構造体(`config`)を
//! モジュールとして公開し，バイナリ(`axelrod`)と統合テストの双方から利用する．

pub mod config;
pub mod world;
pub mod mechanisms;
pub mod simulation;
pub mod metrics;
