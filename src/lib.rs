//! # Typelude
//!
//! 型レベルプログラミングのためのライブラリ。
//!
//! ## Features
//! - `Evaluable` トレイトによる統一的な評価インターフェース
//! - `EApply<F, A>` パターンによる関数と式の分離
//! - 型レベルブール値と論理演算
//! - 型レベル配列と配列操作
//!
//! ## Quick Start
//! ```ignore
//! use typelude::prelude::*;
//!
//! type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
//! // Result = i32
//! ```

#![feature(specialization)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub mod eval;
pub mod func;
pub mod machine;
pub mod prelude;
pub mod types;
