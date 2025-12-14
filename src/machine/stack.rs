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
use crate::func::{EConcat, EFunction, FIsEmpty, FNot};
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

/// Dup: スタックの先頭要素を複製する
#[derive(Debug, Clone, Copy)]
pub struct OpDup;
impl Sealed for OpDup {}
impl Instruction for OpDup {}

/// Swap: スタックの先頭2つの要素を入れ替える
#[derive(Debug, Clone, Copy)]
pub struct OpSwap;
impl Sealed for OpSwap {}
impl Instruction for OpSwap {}

/// Drop: スタックの先頭要素を破棄する
#[derive(Debug, Clone, Copy)]
pub struct OpDrop;
impl Sealed for OpDrop {}
impl Instruction for OpDrop {}

/// If: スタックの先頭がTrueならThen、FalseならElseを実行する
/// - ThenProg: Trueの場合に実行する命令列
/// - ElseProg: Falseの場合に実行する命令列
#[derive(Debug, Clone, Copy)]
pub struct OpIf<ThenProg, ElseProg>(PhantomData<(ThenProg, ElseProg)>);
impl<T, E> Sealed for OpIf<T, E> {}
impl<T, E> Instruction for OpIf<T, E> {}

// =============================================================================
// Instruction Logic (Execute)
// =============================================================================

/// 命令を実行して新しい状態を返すトレイト
///
/// `RestProg` は、現在の命令を取り除いた残りの命令列。
/// 通常の命令は `Execute::OutputState = State<NewStack, RestProg>` となる。
/// 分岐命令は `Execute::OutputState = State<NewStack, NewProg>` となる。
pub trait Execute<Stack, RestProg> {
    type OutputState;
}

/// Helper trait for Stack-only instructions (Adapter Pattern)
pub trait RunStep<Stack> {
    type OutputStack: Cons;
}

// Adapt RunStep to Execute
impl<Inst, Stack, RestProg> Execute<Stack, RestProg> for Inst
where
    Inst: RunStep<Stack>,
    RestProg: Cons,
{
    type OutputState = State<<Inst as RunStep<Stack>>::OutputStack, RestProg>;
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

// --- OpDup ---
// Stack: [A, ...] -> [A, A, ...]
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpDup
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
{
    type OutputStack = TyArray<A, TyArray<A, Rest>>;
}

// --- OpSwap ---
// Stack: [A, B, ...] -> [B, A, ...]
impl<A, B, Rest> RunStep<TyArray<A, TyArray<B, Rest>>> for OpSwap
where
    TyArray<A, TyArray<B, Rest>>: Cons,
    Rest: Cons,
{
    type OutputStack = TyArray<B, TyArray<A, Rest>>;
}

// --- OpDrop ---
// Stack: [A, ...] -> [...]
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpDrop
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
{
    type OutputStack = Rest;
}

// --- OpIf<Then, Else> ---
// Execute for OpIf
// Stack: [Cond, RestStack...]
// if Cond == True  -> State<RestStack, Then + RestProg>
// if Cond == False -> State<RestStack, Else + RestProg>

impl<Cond, RestStack, Then, Else, RestProg> Execute<TyArray<Cond, RestStack>, RestProg>
    for OpIf<Then, Else>
where
    TyArray<Cond, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Then: Cons,
    Else: Cons,
    EConcat<Then, RestProg>: Evaluable,
    EConcat<Else, RestProg>: Evaluable,
    EIf<Cond, State<RestStack, Evaluator<EConcat<Then, RestProg>>>, State<RestStack, Evaluator<EConcat<Else, RestProg>>>>: Evaluable,
{
    type OutputState = Evaluator<
        EIf<
            Cond,
            State<RestStack, Evaluator<EConcat<Then, RestProg>>>,
            State<RestStack, Evaluator<EConcat<Else, RestProg>>>,
        >,
    >;
}

// =============================================================================
// Machine Execution (Step / Run)
// =============================================================================

/// 1ステップ実行
/// State<Stack, Cons<Inst, RestProg>> -> State<NewStack, RestProg>
pub struct FStep;

impl<Stack, Inst, RestProg> EFunction<State<Stack, TyArray<Inst, RestProg>>> for FStep
where
    Inst: Execute<Stack, RestProg>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    type Output = <Inst as Execute<Stack, RestProg>>::OutputState;
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

    #[test]
    fn test_op_if() {
        use crate::types::bool::{TyFalse, TyTrue};

        // If True then Push 1 else Push 2
        type IfProg = OpIf<tyarray![OpPush<U1>], tyarray![OpPush<U2>]>;

        // Case 1: True
        type Prog1 = tyarray![OpPush<TyTrue>, IfProg];
        type FinalState1 = Evaluator<ERun<State<TyNil, Prog1>>>;
        assert_type_eq_all!(FinalState1, State<tyarray![U1], TyNil>);

        // Case 2: False
        type Prog2 = tyarray![OpPush<TyFalse>, IfProg];
        type FinalState2 = Evaluator<ERun<State<TyNil, Prog2>>>;
        assert_type_eq_all!(FinalState2, State<tyarray![U2], TyNil>);
    }
}
