//! **Type-Level Stack Machine**
//!
//! 型レベルで動作するシンプルなスタックマシンの実装。
//!
//! # Architecture
//! - **Stack**: 値 (`typenum`) のリスト (`TyArray`)
//! - **Instruction**: 操作を表す型 (`OpPush`, `OpAdd`, etc.)
//! - **Program**: 命令のリスト (`TyArray`)
//! - **State**: (Stack, Program) のペア
//! - **Step**: 1ステップ実行する関数
//! - **Run**: プログラムが空になるまで実行する関数

use std::marker::PhantomData;

use crate::eval::{EApply, EIf, EWhile, Evaluable, Evaluator, Sealed};
use crate::func::{EFunction, FIsEmpty, FNot};
use crate::types::array::{Cons, Head, Tail, TyArray, TyNil};

// =============================================================================
// Machine State
// =============================================================================

/// マシンの状態: (Stack, Program)
/// - Stack: 計算スタック (TyArray)
/// - Program: 実行待ち命令リスト (TyArray<Instruction>)
#[derive(Debug)]
pub struct State<Stack, Program>(PhantomData<(Stack, Program)>);

impl<S, P> Sealed for State<S, P> {}

impl<S, P> Evaluable for State<S, P> {
    type Output = State<S, P>;
}

// =============================================================================
// Instructions
// =============================================================================

/// 命令マーカー
pub trait Instruction: Sealed {}

/// Push N: スタックに値 N を積む
#[derive(Debug, Clone, Copy)]
pub struct OpPush<N>(PhantomData<N>);
impl<N> Sealed for OpPush<N> {}
impl<N> Instruction for OpPush<N> {}

/// Add: スタックの上位2つを取り出し、足して積む
#[derive(Debug, Clone, Copy)]
pub struct OpAdd;
impl Sealed for OpAdd {}
impl Instruction for OpAdd {}

/// Sub: スタックの上位2つを取り出し、引いて積む
#[derive(Debug, Clone, Copy)]
pub struct OpSub;
impl Sealed for OpSub {}
impl Instruction for OpSub {}

// =============================================================================
// Instruction Logic (RunStep)
// =============================================================================

/// 命令を実行して新しいスタックを返すトレイト
pub trait RunStep<Stack> {
    type OutputStack: Cons;
}

// --- OpPush<N> ---
// Stack -> Cons<N, Stack>
impl<N, Stack> RunStep<Stack> for OpPush<N>
where
    Stack: Cons,
{
    type OutputStack = TyArray<N, Stack>;
}

// --- OpAdd ---
// Stack: [B, A, ...] -> Push A + B
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpAdd
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::func::EAdd<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::func::EAdd<A, B>>, Rest>;
}

// --- OpSub ---
// Stack: [B, A, ...] -> Push A - B
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpSub
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::func::ESub<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::func::ESub<A, B>>, Rest>;
}

// =============================================================================
// Machine Execution (Step / Run)
// =============================================================================

/// 1ステップ実行
/// State<Stack, Cons<Inst, RestProg>> -> State<NewStack, RestProg>
pub struct FStep;

impl<Stack, Inst, RestProg> EFunction<State<Stack, TyArray<Inst, RestProg>>> for FStep
where
    Inst: RunStep<Stack>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    type Output = State<<Inst as RunStep<Stack>>::OutputStack, RestProg>;
}

// Evaluable wrapper for FStep
impl<S, P> Evaluable for EApply<FStep, State<S, P>>
where
    FStep: EFunction<State<S, P>>,
{
    type Output = <FStep as EFunction<State<S, P>>>::Output;
}

// --- IsFinished: プログラムが空か判定 ---
pub struct FIsFinished;

impl<S, P> EFunction<State<S, P>> for FIsFinished
where
    EApply<FIsEmpty, P>: Evaluable,
    Evaluator<EApply<FIsEmpty, P>>: crate::types::bool::_NotHelper,
{
    type Output = Evaluator<EApply<FNot, EApply<FIsEmpty, P>>>;
}

// --- Machine Runner ---
// Run<InitialState> -> FinalState
// uses EWhile<Condition, Step, State>

pub type ERun<S> = EWhile<FIsFinished, FStep, S>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tyarray;
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2, U3, U5};

    #[test]
    fn test_stack_ops() {
        type S0 = TyNil;

        // Push 1 -> [1]
        type S1 = <OpPush<U1> as RunStep<S0>>::OutputStack;
        assert_type_eq_all!(S1, tyarray![U1]);

        // Push 2 -> [2, 1]
        type S2 = <OpPush<U2> as RunStep<S1>>::OutputStack;
        assert_type_eq_all!(S2, tyarray![U2, U1]);

        // Add -> [1+2] = [3]
        type S3 = <OpAdd as RunStep<S2>>::OutputStack;
        assert_type_eq_all!(S3, tyarray![U3]);
    }

    #[test]
    fn test_machine_run() {
        // Program: Push 2, Push 3, Add, Push 5, Sub
        // [2] -> [3, 2] -> [5] -> [5, 5] -> [0]

        type Prog = tyarray![
            OpPush<U2>,
            OpPush<U3>,
            OpAdd,
            OpPush<U5>,
            OpSub
        ];

        type InitialState = State<TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;

        // FinalState should be State<[0], []>
        // But let's check the stack only.

        // We can't access fields of State directly easily without helper traits,
        // but we can assert the type.
        type ExpectedStack = tyarray![typenum::U0];
        type ExpectedState = State<ExpectedStack, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
