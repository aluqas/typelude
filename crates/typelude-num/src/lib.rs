//! 型レベル数値プリミティブと算術演算。
//!
//! ## 責務
//!
//! ペアノ数（Zero、Succ）による型レベル自然数と
//! 算術能力トレイト（Add、Sub、Mul、Div）を提供。
//! typenum を再エクスポート（相互運用性確保）。
//!
//! ## 名前空間マッピング
//!
//! - `Add` -> `OpAdd`
//! - `Sub` -> `OpSub`
//! - `Mul` -> `OpMul`
//! - `Div` -> `OpDiv`

#![recursion_limit = "256"]

pub use typelude_std::core::{Add, Div, Mul, Sub};
pub use typenum::*;
extern crate typenum;

pub mod peano;
