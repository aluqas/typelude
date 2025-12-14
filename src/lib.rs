//! # Typelude
//!
//! 型レベルプログラミングのためのライブラリ。
//!
//! ## Features
//!
//! - `Evaluable` トレイトによる統一的な評価インターフェース
//! - `EApply<F, A>` パターンによる関数と式の分離
//! - 型レベルブール値と論理演算
//! - 型レベル配列と配列操作
//!
//! ## Quick Start
//!
//! ```ignore
//! use typelude::prelude::*;
//!
//! // 条件分岐
//! type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
//! // Result = i32
//!
//! // 配列操作
//! type Len = Evaluator<ELen<ELit<tyarray![i32, f64, bool]>>>;
//! // Len = U3
//! ```
//!
//! ## Architecture
//!
//! ```text
//! eval/       - 評価インフラ (Evaluable, ELit, EIf, EWhile, EApply)
//! func/       - 関数マーカー (FNot, FAnd, FLen, etc.)
//! types/      - データ型 (TyTrue/TyFalse, TyArray/TyNil)
//! prelude     - 便利な一括 import
//! ```

// Unstable features required for type-level programming
#![feature(specialization)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub mod eval;
pub mod func;
pub mod machine;
pub mod prelude;
pub mod types;
