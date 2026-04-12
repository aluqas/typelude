//! Typeludeの標準ライブラリ。
//!
//! 型レベル計算の基盤（評価エンジン、高階演算子）と、
//! 標準的な計算構造（制御フロー、エフェクト、値変換）を提供する。
//!
//! ## 主要モジュール
//!
//! - **core**: 評価基盤（Eval、高階演算子Op、値変換）
//! - **control**: 制御フロー（If、While）
//! - **effect**: エフェクトモナド（Reader、Writer、Suspend）
//! - **debug**: デバッグ・検証ユーティリティ
//! - **macros**: DSL・プロシージャルマクロサポート
//!
//! ## 設計パターン
//!
//! 型レベル計算は Eval トレイトで統一され、型エイリアスと異なり
//! 再帰型対応と制約内部化により複雑な計算を暗く実装できる。

// Unstable features required for type-level programming

// Use for NegativeEquality
#![cfg_attr(feature = "nightly", feature(specialization))]
// Use for Integration to value
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "65536"]

extern crate self as typelude_std;

#[macro_use]
pub mod macros;
pub mod control;
pub mod core;
pub mod debug;
pub mod effect;

pub use core::{Apply, Eval, Evaluate, IntoValue, Lift, Op, Reflect, Reify, Value};

pub use control::{If, While};
