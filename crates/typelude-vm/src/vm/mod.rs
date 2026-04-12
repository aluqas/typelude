//! VM 実行層：プロトコル、純粋意味論、ランタイム、妥当性検証。
//!
//! ## 層構成
//!
//! - **protocol**: エラー型・status型・値型の定義
//! - **semantics**: 純粋な状態遷移関数（1ステップ実行）
//! - **value**: スタック値・メモリ表現
//! - **runtime**: エフェクト付き実行ループ（モナド変換子）
//! - **well_formed**: 型レベル妥当性検証（静的証明）

pub mod protocol;
pub mod runtime;
pub mod semantics;
pub mod value;
pub mod well_formed;
