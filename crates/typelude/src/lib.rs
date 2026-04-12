#![recursion_limit = "16384"]
//! # Typelude
//!
//! Rustの型システムで型レベル計算を行うための統合フレームワーク。
//! 型をファーストクラス値として扱い、コンパイル時に任意の計算
//! を実現する。
//!
//! ## 主要モジュール
//!
//! - **core**: 型式評価の基盤（Eval、高階演算子）
//! - **std**: 型レベル標準ライブラリ（配列・マップ・ツリー、制御フロー）
//! - **vm**: 型レベルスタックマシン（命令、実行意味論）
//! - **wasm**: WebAssembly型レベル表現（オペコード、フレーム）
//! - **Macros**: DSL群（proc_macro展開）
//!
//! ## 設計パターン
//!
//! 型レベル計算は Eval トレイトで統一インターフェース化する。

pub mod core {
    pub use tstr;
    pub use typelude_std::core::*;
    pub use typenum;
}
pub use tstr;
pub use typelude_macros::twat;
pub use typelude_std::{Eval, Evaluate};
pub use typelude_vm as vm;
pub use typelude_wasm as wasm;
pub use typenum;
