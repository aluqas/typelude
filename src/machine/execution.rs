//! **Type-Level Stack Machine Execution Logic**
//!
//! マシンの命令実行ロジックとメインループを実装します。

use crate::eval::{EApply, EIf, EWhile, Evaluable, Evaluator};
use crate::func::{EConcat, EFunction, FIsEmpty, FNot};
// Important: Ensure types::int is imported so Evaluable impls for Arithmetic are visible
#[allow(unused_imports)]
use crate::types::int;
use crate::machine::instruction::*;
use crate::machine::state::MachineState;
use crate::types::array::{Cons, Get, Set, TyArray, TyNil};
use typenum::Unsigned;

// =============================================================================
// Execute Trait
// =============================================================================

/// 命令を実行して新しい状態を返すトレイト
///
/// `RestProg` は、現在の命令を取り除いた残りの命令列。
pub trait Execute<Stack, Memory, CallStack, RestProg> {
    type OutputState;
}

// --- Helper trait for Stack-only instructions (Adapter Pattern) ---
pub trait RunStep<Stack> {
    type OutputStack: Cons;
}

// Note: Removed generic implementation of Execute for RunStep to avoid conflict with OpCall/OpReturn/OpIf.
// Instead, we will implement Execute for each Stack-only instruction using a macro or explicitly.

macro_rules! impl_execute_via_runstep {
    ($Inst:ty) => {
        impl<Stack, Memory, CallStack, RestProg> Execute<Stack, Memory, CallStack, RestProg> for $Inst
        where
            Self: RunStep<Stack>,
            RestProg: Cons,
        {
            type OutputState = MachineState<<Self as RunStep<Stack>>::OutputStack, Memory, CallStack, RestProg>;
        }
    };
    ($Inst:ident < $($T:ident),+ >) => {
        impl<$($T),+, Stack, Memory, CallStack, RestProg> Execute<Stack, Memory, CallStack, RestProg> for $Inst<$($T),+>
        where
            Self: RunStep<Stack>,
            RestProg: Cons,
        {
            type OutputState = MachineState<<Self as RunStep<Stack>>::OutputStack, Memory, CallStack, RestProg>;
        }
    };
}

impl<N, Stack, Memory, CallStack, RestProg> Execute<Stack, Memory, CallStack, RestProg> for OpPush<N>
where
    Self: RunStep<Stack>,
    RestProg: Cons,
{
    type OutputState = MachineState<<Self as RunStep<Stack>>::OutputStack, Memory, CallStack, RestProg>;
}
impl_execute_via_runstep!(OpAdd);
impl_execute_via_runstep!(OpSub);
impl_execute_via_runstep!(OpDup);
impl_execute_via_runstep!(OpSwap);
impl_execute_via_runstep!(OpDrop);

// =============================================================================
// Instruction Implementations
// =============================================================================

// --- OpPush<N> ---
impl<N, Stack> RunStep<Stack> for OpPush<N>
where
    Stack: Cons,
{
    type OutputStack = TyArray<N, Stack>;
}

// --- OpAdd ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpAdd
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::types::int::EAdd<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::types::int::EAdd<A, B>>, Rest>;
}

// --- OpSub ---
impl<A, B, Rest> RunStep<TyArray<B, TyArray<A, Rest>>> for OpSub
where
    TyArray<B, TyArray<A, Rest>>: Cons,
    Rest: Cons,
    crate::types::int::ESub<A, B>: Evaluable,
{
    type OutputStack = TyArray<Evaluator<crate::types::int::ESub<A, B>>, Rest>;
}

// --- OpDup ---
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpDup
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
{
    type OutputStack = TyArray<A, TyArray<A, Rest>>;
}

// --- OpSwap ---
impl<A, B, Rest> RunStep<TyArray<A, TyArray<B, Rest>>> for OpSwap
where
    TyArray<A, TyArray<B, Rest>>: Cons,
    Rest: Cons,
{
    type OutputStack = TyArray<B, TyArray<A, Rest>>;
}

// --- OpDrop ---
impl<A, Rest> RunStep<TyArray<A, Rest>> for OpDrop
where
    TyArray<A, Rest>: Cons,
    Rest: Cons,
{
    type OutputStack = Rest;
}

// --- OpLoad ---
// Stack: [Addr, ...] -> Memory, ... -> Stack: [Value, ...]
impl<Addr, RestStack, Memory, CallStack, RestProg> Execute<TyArray<Addr, RestStack>, Memory, CallStack, RestProg> for OpLoad
where
    Addr: Unsigned,
    TyArray<Addr, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Get<Addr>,
{
    type OutputState = MachineState<TyArray<<Memory as Get<Addr>>::Output, RestStack>, Memory, CallStack, RestProg>;
}

// --- OpStore ---
// Stack: [Value, Addr, ...] -> Memory, ... -> Memory[Addr] = Value, Stack: [...]
impl<Value, Addr, RestStack, Memory, CallStack, RestProg> Execute<TyArray<Value, TyArray<Addr, RestStack>>, Memory, CallStack, RestProg> for OpStore
where
    Addr: Unsigned,
    TyArray<Value, TyArray<Addr, RestStack>>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Set<Addr, Value>,
{
    type OutputState = MachineState<RestStack, <Memory as Set<Addr, Value>>::Output, CallStack, RestProg>;
}

// --- OpCall<TargetProg> ---
// CallStack: [...] -> [RestProg, ...]
// Program: TargetProg
impl<TargetProg, Stack, Memory, CallStack, RestProg> Execute<Stack, Memory, CallStack, RestProg> for OpCall<TargetProg>
where
    Stack: Cons,
    Memory: Cons,
    CallStack: Cons,
    TargetProg: Cons,
    RestProg: Cons,
{
    type OutputState = MachineState<Stack, Memory, TyArray<RestProg, CallStack>, TargetProg>;
}

// --- OpReturn ---
// CallStack: [Continuation, RestCallStack...] -> RestCallStack
// Program: Continuation
impl<Stack, Memory, Continuation, RestCallStack, RestProg> Execute<Stack, Memory, TyArray<Continuation, RestCallStack>, RestProg> for OpReturn
where
    Stack: Cons,
    Memory: Cons,
    TyArray<Continuation, RestCallStack>: Cons,
    Continuation: Cons,
    RestCallStack: Cons,
    RestProg: Cons, // Note: RestProg (current prog) is discarded or expected to be empty/just Return
{
    type OutputState = MachineState<Stack, Memory, RestCallStack, Continuation>;
}

// --- OpIf<Then, Else> ---
impl<Cond, RestStack, Memory, CallStack, Then, Else, RestProg> Execute<TyArray<Cond, RestStack>, Memory, CallStack, RestProg>
    for OpIf<Then, Else>
where
    TyArray<Cond, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Then: Cons,
    Else: Cons,
    EConcat<Then, RestProg>: Evaluable,
    EConcat<Else, RestProg>: Evaluable,
    EIf<
        Cond,
        MachineState<RestStack, Memory, CallStack, Evaluator<EConcat<Then, RestProg>>>,
        MachineState<RestStack, Memory, CallStack, Evaluator<EConcat<Else, RestProg>>>,
    >: Evaluable,
{
    type OutputState = Evaluator<
        EIf<
            Cond,
            MachineState<RestStack, Memory, CallStack, Evaluator<EConcat<Then, RestProg>>>,
            MachineState<RestStack, Memory, CallStack, Evaluator<EConcat<Else, RestProg>>>,
        >,
    >;
}

// =============================================================================
// Machine Execution (Step / Run)
// =============================================================================

/// 1ステップ実行
/// MachineState<Stack, Memory, CallStack, Cons<Inst, RestProg>> -> NewState
pub struct FStep;

impl<Stack, Memory, CallStack, Inst, RestProg> EFunction<MachineState<Stack, Memory, CallStack, TyArray<Inst, RestProg>>> for FStep
where
    Inst: Execute<Stack, Memory, CallStack, RestProg>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    type Output = <Inst as Execute<Stack, Memory, CallStack, RestProg>>::OutputState;
}

// Evaluable wrapper for FStep
impl<S, M, C, P> Evaluable for EApply<FStep, MachineState<S, M, C, P>>
where
    FStep: EFunction<MachineState<S, M, C, P>>,
{
    type Output = <FStep as EFunction<MachineState<S, M, C, P>>>::Output;
}

// --- IsFinished: プログラムが空か判定 ---
pub struct FIsFinished;

impl<S, M, C, P> EFunction<MachineState<S, M, C, P>> for FIsFinished
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
    fn test_stack_ops_logic() {
        type S0 = TyNil;
        // Push 1
        type S1 = <OpPush<U1> as RunStep<S0>>::OutputStack;
        assert_type_eq_all!(S1, tyarray![U1]);
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

        // Empty Memory, Empty CallStack
        type InitialState = MachineState<TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U0];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_call_return() {
        // Subroutine: Push 5, Return
        type SubRoutine = tyarray![
            OpPush<U5>,
            OpReturn
        ];

        // Main: Push 3, Call SubRoutine, Add
        // [3] -> Call -> [5, 3] -> [8]
        type MainProg = tyarray![
            OpPush<U3>,
            OpCall<SubRoutine>,
            OpAdd
        ];

        type InitialState = MachineState<TyNil, TyNil, TyNil, MainProg>;
        type FinalState = Evaluator<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U8];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_memory_and_function() {
        use typenum::{U10, U20};

        // Initialize memory with 0 at address 0
        type InitialMemory = tyarray![typenum::U0];

        // Function: Increment value at address 0
        // Stack: []
        // Logic: Push 0 (addr), Push 10 (val), Store
        //        Push 0 (addr), Load, Push 20 (val), Add
        //        Push 0 (addr), Swap, Store, Return
        type Func = tyarray![
            OpPush<typenum::U0>, // Addr
            OpPush<U10>,         // Val
            OpStore,             // [Val, Addr] -> Mem[Addr]=Val. Stack: []
            OpPush<typenum::U0>, // Addr
            OpLoad,              // [Addr] -> [Val] (10)
            OpPush<U20>,         // [20, 10]
            OpAdd,               // [30]
            OpPush<typenum::U0>, // [0, 30] (Addr, Val)
            OpSwap,              // [30, 0] (Val, Addr)
            OpStore,             // Mem[0]=30. Stack: []
            OpReturn
        ];

        // Main: Call Func
        type Main = tyarray![
            OpCall<Func>
        ];

        type InitialState = MachineState<TyNil, InitialMemory, TyNil, Main>;
        type FinalState = Evaluator<ERun<InitialState>>;

        // Expected Memory: [30] (10 + 20)
        type ExpectedMemory = tyarray![typenum::U30];
        type ExpectedState = MachineState<TyNil, ExpectedMemory, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
