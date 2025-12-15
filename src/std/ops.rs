//! **General Operators**
//!
//! 汎用的な操作マーカー（`FEq`など）を定義します。
//! 特定の型に依存しない、または複数の型で共有される概念です。

use crate::eval::ELit;

// =============================================================================
// EFunction Trait
// =============================================================================

/// 関数を表すトレイト
///
/// `EWhile` などで使用される、型から型への変換を表します。
pub trait EFunction<A> {
    type Output;
}

// ELit を自動的にアンラップ
impl<F, T> EFunction<ELit<T>> for F
where
    F: EFunction<T>,
{
    type Output = <F as EFunction<T>>::Output;
}
