//! **Type Equality**
//!
//! 型レベルでの等価性判定を提供します。

use crate::{
    eval::{EApply2, Evaluable, Evaluator},
    func::{FEq, FNotEq},
    types::bool::{TyFalse, TyTrue},
};

// =============================================================================
// Type Equality Helper (using generic_const_exprs)
// =============================================================================

/// 型の等価性判定ヘルパー (associated const版)
/// `default const` は具体値として解決されるため、specializationが正しく動作する
#[doc(hidden)]
pub trait _TypeEqConst<A, B> {
    const ARE_EQUAL: bool;
}

impl<A, B> _TypeEqConst<A, B> for () {
    default const ARE_EQUAL: bool = false;
}

impl<T> _TypeEqConst<T, T> for () {
    const ARE_EQUAL: bool = true;
}

/// const bool から TyTrue/TyFalse への変換
pub struct AssertBool<const COND: bool>;

impl Evaluable for AssertBool<true> {
    type Output = TyTrue;
}

impl Evaluable for AssertBool<false> {
    type Output = TyFalse;
}

// =============================================================================
// FEq / FNotEq Implementation
// =============================================================================

/// FEq: 型の等価性
impl<L, R> Evaluable for EApply2<FEq, L, R>
where
    L: Evaluable,
    R: Evaluable,
    (): _TypeEqConst<Evaluator<L>, Evaluator<R>>,
    AssertBool<{ <() as _TypeEqConst<Evaluator<L>, Evaluator<R>>>::ARE_EQUAL }>: Evaluable,
{
    type Output =
        Evaluator<AssertBool<{ <() as _TypeEqConst<Evaluator<L>, Evaluator<R>>>::ARE_EQUAL }>>;
}

/// FNotEq: 型の非等価性
impl<L, R> Evaluable for EApply2<FNotEq, L, R>
where
    L: Evaluable,
    R: Evaluable,
    (): _TypeEqConst<Evaluator<L>, Evaluator<R>>,
    AssertBool<{ !<() as _TypeEqConst<Evaluator<L>, Evaluator<R>>>::ARE_EQUAL }>: Evaluable,
{
    type Output =
        Evaluator<AssertBool<{ !<() as _TypeEqConst<Evaluator<L>, Evaluator<R>>>::ARE_EQUAL }>>;
}
