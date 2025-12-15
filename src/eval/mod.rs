//! **Evaluation Infrastructure**
//!
//! 型レベル計算のコアインフラを提供します。

mod expr;

pub use expr::*;

// =============================================================================
// Sealed Trait (for internal use)
// =============================================================================

/// Sealed trait pattern - 外部クレートからの実装を防ぐ
///
/// このトレイトは内部実装用トレイトであり、直接使用しないでください。
#[doc(hidden)]
pub trait Sealed {}

// =============================================================================
// Core Evaluable Trait
// =============================================================================

/// 型レベル式を評価するトレイト
///
/// `Evaluable` を実装した型は `Evaluator<T>` で評価結果を取得できます。
pub trait Evaluable {
    type Output;
}

/// 式の評価結果を取得する型エイリアス
///
/// # Example
/// ```ignore
/// type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
/// // Result = i32
/// ```
pub type Evaluator<T> = <T as Evaluable>::Output;
