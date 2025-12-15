//! **Expression Types**
//!
//! 型レベル式の構文を定義します。
//! - `ELit`: リテラル
//! - `EApply`, `EApply2`: 関数適用
//! - `EIf`: 条件分岐
//! - `EWhile`: ループ

use std::marker::PhantomData;

use super::{Evaluable, Evaluator};
use crate::{
    std::bool::{TyFalse, TyTrue},
    std::ops::EFunction,
};

// =============================================================================
// ELit: Literal Expression
// =============================================================================

/// リテラル式 - 通常の型を式として埋め込む
///
/// # Example
/// ```ignore
/// type Five = ELit<typenum::U5>;
/// assert_type_eq!(Evaluator<Five>, typenum::U5);
/// ```
pub struct ELit<T>(PhantomData<T>);

impl<T> Evaluable for ELit<T> {
    type Output = T;
}

// =============================================================================
// EApply: Function Application
// =============================================================================

/// 1引数関数適用
///
/// # Example
/// ```ignore
/// type LenExpr = EApply<FLen, ELit<MyArray>>;
/// ```
pub struct EApply<F, A>(PhantomData<(F, A)>);

/// 2引数関数適用
///
/// # Example
/// ```ignore
/// type ConcatExpr = EApply2<FConcat, ELit<ArrayA>, ELit<ArrayB>>;
/// ```
pub struct EApply2<F, A, B>(PhantomData<(F, A, B)>);

/// 3引数関数適用
///
/// # Example
/// ```ignore
/// type SetExpr = EApply3<FSet, ELit<Array>, ELit<Index>, ELit<Value>>;
/// ```
pub struct EApply3<F, A, B, C>(PhantomData<(F, A, B, C)>);

// =============================================================================
// EIf: Conditional Expression
// =============================================================================

/// 条件分岐式
///
/// # Example
/// ```ignore
/// type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
/// // Result = i32
/// ```
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

/// Condの結果（True/False）に対して実装するヘルパー
#[doc(hidden)]
pub trait _EIfHelper<Then, Else> {
    type Output;
}

impl<Then, Else> _EIfHelper<Then, Else> for TyTrue
where
    Then: Evaluable,
{
    type Output = Then::Output;
}

impl<Then, Else> _EIfHelper<Then, Else> for TyFalse
where
    Else: Evaluable,
{
    type Output = Else::Output;
}

impl<Cond, Then, Else> Evaluable for EIf<Cond, Then, Else>
where
    Cond: Evaluable,
    Evaluator<Cond>: _EIfHelper<Then, Else>,
{
    type Output = <Evaluator<Cond> as _EIfHelper<Then, Else>>::Output;
}

// =============================================================================
// EWhile: Loop Expression
// =============================================================================

type AppliedOutput<F, A> = <F as EFunction<A>>::Output;

/// Whileループを表す式
///
/// - `Pred`: 継続条件 (State → TyTrue/TyFalse)
/// - `Step`: 更新関数 (State → NextState)
/// - `State`: 現在の状態
///
/// # Example
/// ```ignore
/// // while (x < 10) { x = x + 1 }
/// type Result = Evaluator<EWhile<IsLessThan10, PlusOne, U1>>;
/// // Result = U10
/// ```
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);

#[doc(hidden)]
pub trait _EWhileHelper<Pred, Step, State> {
    type Output;
}

// Condition == True: 再帰
impl<Pred, Step, State> _EWhileHelper<Pred, Step, State> for TyTrue
where
    Step: EFunction<State>,
    Step::Output: Evaluable,
    EWhile<Pred, Step, AppliedOutput<Step, State>>: Evaluable,
{
    type Output = <EWhile<Pred, Step, AppliedOutput<Step, State>> as Evaluable>::Output;
}

// Condition == False: 終了
impl<Pred, Step, State> _EWhileHelper<Pred, Step, State> for TyFalse
where
    State: Evaluable,
{
    type Output = Evaluator<State>;
}

impl<Pred, Step, State> Evaluable for EWhile<Pred, Step, State>
where
    Pred: EFunction<State>,
    AppliedOutput<Pred, State>: Evaluable,
    Evaluator<AppliedOutput<Pred, State>>: _EWhileHelper<Pred, Step, State>,
{
    type Output =
        <Evaluator<AppliedOutput<Pred, State>> as _EWhileHelper<Pred, Step, State>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::{assert_type_eq_all, assert_type_ne_all};

    use crate::{
        eval::{EIf, ELit, EWhile, Evaluator},
        std::{
            array::{Cons, TyArray, TyNil},
            bool::{ToTyBool, ToTyBoolOut, TyFalse, TyTrue},
            ops::EFunction,
        },
    };

    #[test]
    fn test_eval_if() {
        assert_type_eq_all!(Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>, i32);
        assert_type_eq_all!(Evaluator<EIf<ELit<TyFalse>, ELit<i32>, ELit<f64>>>, f64);
        assert_type_ne_all!(Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<()>>>, ());
    }

    #[test]
    fn test_eval_while() {
        // Condition: IsNotEmpty
        struct IsNotEmpty;
        impl EFunction<TyNil> for IsNotEmpty {
            type Output = TyFalse;
        }
        impl<H, T> EFunction<TyArray<H, T>> for IsNotEmpty
        where
            T: Cons,
        {
            type Output = TyTrue;
        }

        // Step: GetTail
        struct GetTail;
        impl<H, T> EFunction<TyArray<H, T>> for GetTail
        where
            T: Cons,
        {
            type Output = ELit<T>;
        }

        assert_type_eq_all!(
            Evaluator<EWhile<IsNotEmpty, GetTail, TyArray<i32, TyArray<f64, TyArray<(), TyNil>>>>>,
            TyNil
        );
    }

    #[test]
    fn while_loop_plus_one() {
        use typenum::{Add1, IsLess, U1, U10, Unsigned};

        struct IsLessThan10;
        impl<T> EFunction<T> for IsLessThan10
        where
            T: IsLess<U10>,
            <T as IsLess<U10>>::Output: ToTyBool,
        {
            type Output = ToTyBoolOut<<T as IsLess<U10>>::Output>;
        }

        struct PlusOne;
        impl<T> EFunction<T> for PlusOne
        where
            T: std::ops::Add<typenum::B1>,
            Add1<T>: Unsigned,
        {
            type Output = ELit<Add1<T>>;
        }

        type Result = Evaluator<EWhile<IsLessThan10, PlusOne, U1>>;
        assert_type_eq_all!(Result, U10);
    }
}
