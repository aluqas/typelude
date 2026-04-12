//! 自己評価値とタプル引数パック。
//!
//! Value トレイトレンにより、値型の自己評価と、
//! 複数引数をタプルで表現する機構を提供。

use super::{Eval, Evaluate};

/// 自分自身へ評価される値の型マーカートレイト。
///
/// Value を実装した型 T は自動的に Eval も実装し、
/// Evaluate<T> = T として評価される。これにより、
/// リテラル値（Zero、True等）や複合値（タプル）が
/// 型レベル式の安全な基底ケースとなる。
pub trait Value {}

impl<T> Eval for T
where
    T: Value,
{
    type Output = Self;
}

impl Eval for () {
    type Output = ();
}

impl<A> Eval for (A,)
where
    A: Eval,
{
    type Output = (Evaluate<A>,);
}

impl<A, B> Eval for (A, B)
where
    A: Eval,
    B: Eval,
{
    type Output = (Evaluate<A>, Evaluate<B>);
}

impl<A, B, C> Eval for (A, B, C)
where
    A: Eval,
    B: Eval,
    C: Eval,
{
    type Output = (Evaluate<A>, Evaluate<B>, Evaluate<C>);
}

impl<A, B, C, D> Eval for (A, B, C, D)
where
    A: Eval,
    B: Eval,
    C: Eval,
    D: Eval,
{
    type Output = (Evaluate<A>, Evaluate<B>, Evaluate<C>, Evaluate<D>);
}
