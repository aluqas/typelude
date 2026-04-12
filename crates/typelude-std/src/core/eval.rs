//! 型レベル式評価システムの境界。
//!
//! Eval トレイトにより、あらゆる型レベル計算式を統一的に評価できる。
//! 型エイリアスではなくトレイト手法を使うことで、再帰型対応と制約内部化を実現。

/// 型レベル式評価の共通インターフェース。
///
/// 型式 T が Eval を実装していれば、`Evaluate<T>` により
/// その計算結果である Output 型へ評価される。
///
/// # 設計の核心
///
/// 型エイリアスとの差別化：
/// - **再帰型対応**: `type Fib<N> = Add<Fib<Pred<N>>, ...>`
///   が型エイリアスでは不可だが、 Eval パターンでは `impl Eval for EFib<N>`
///   に制約を内部化するため可能。
/// - **制約内部化**: ジェネリクスの外側では `EFoo<A, B>: Eval` の一言で済み、
///   内部の複雑な制約（A: Bound 等）が呼び出し側に漏れない。
///
/// # 例
///
/// ```ignore
/// // 型レベル加算の定義
/// impl Eval for EAdd<ELit<U3>, ELit<U5>> {
///     type Output = U8;
/// }
///
/// // 評価
/// type Sum = Evaluate<EAdd<ELit<U3>, ELit<U5>>>;  // Sum = U8
/// ```
pub trait Eval {
    /// このEval式が評価される先の型。
    type Output;
}

/// 型レベル式を正規形（Output）へ評価するためのエイリアス。
///
/// `T: Eval` ならば `Evaluate<T>` は `T::Output` と等価。
pub type Evaluate<T> = <T as Eval>::Output;
