//! **Core Traits**
//!
//! typelude の基本的なトレイト定義。

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
