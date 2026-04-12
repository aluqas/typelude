//! 一級演算子と適用表現（AST）。
//!
//! このモジュールは Typelude の正規高階ABI を定義する。
//! プリミティブクレート（typelude-bool、typelude-num、typelude-col）は
//! 値と能力トレイトを提供し、共有高階API は Op<Args> と Apply<F, Args>
//! で通信する。

use core::marker::PhantomData;

use super::{Eval, Evaluate};

/// 高階制御フロー・状態マシン構成で利用可能な一級演算子の共通インターフェース。
///
/// OpAdd や OpGet 等の正規演算子の実装は、意図的に typelude-std が所有する。
/// これにより、各プリミティブクレートは値の表現に集中でき、
/// 高階合成はセントラルな統制の下で行われ、一貫性が保証される。
pub trait Op<Args> {
    /// この演算子が Args に対して計算する結果型。
    type Output;
}

/// 演算子適用を表す型レベルAST ノード。
///
/// F（演算子）に Args（引数）を適用することを型式として記述し、
/// Eval により評価される。
pub struct Apply<F, Args>(pub PhantomData<(F, Args)>);

impl<F, Args> Eval for Apply<F, Args>
where
    Args: Eval,
    F: Op<Evaluate<Args>>,
{
    type Output = <F as Op<Evaluate<Args>>>::Output;
}
