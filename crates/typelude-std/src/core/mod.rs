//! 型レベル実行システムの基盤プリミティブ。
//!
//! ## 概要
//!
//! Eval トレイトと高階演算子（Op）により、
//! 一貫した型レベル評価インターフェースを提供。 値変換（Lift/Reflect/Reify）、
//! 能力トレイト（Add/Get/Set等）を統合し、 再帰型対応と制約内部化を実現。
//!
//! ## 主要概念
//!
//! - **Eval**: 型レベル式の評価トレイト（`type Output`により計算結果を表現）
//! - **Op**: 高階演算子（引数を受け取り適用可能）
//! - **Lift/Reflect/Reify**: 値型と型レベル型間の変換
//! - **Traits**: 能力マーカー（Add/Not/And/Or/Get/Set等）

mod assert;
mod bridge;
mod convert;
mod eval;
mod op;
mod op_adapters;
mod ops;
mod traits;
mod typenum_impls;
mod value;

pub use assert::{Assert, Unwrap, UnwrapOr};
pub use bridge::{IntoValue, Lift, Reflect, Reify};
pub use convert::{From, Into};
pub use eval::{Eval, Evaluate};
pub use op::{Apply, Op, Op as TyFn};
pub use ops::{
    OpAdd, OpAnd, OpConcat, OpDiv, OpEq, OpFold, OpGet, OpGt, OpHead, OpIf, OpInto, OpLe, OpLen,
    OpLt, OpMap, OpMul, OpNand, OpNeq, OpNot, OpOr, OpPow, OpRem, OpSet, OpSub, OpTail, OpWhile,
    OpXor,
};
pub use traits::{
    Add, And, Append, Concat, Div, Eq, Get, Gt, Head, Le, Len, Lt, Mul, Nand, Neq, Not, Or,
    Prepend, Set, Sub, Tail, Xor,
};
pub use value::Value;
