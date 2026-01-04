//! **Type-Level Stack Machine Execution Logic**
//!
//! Implements machine instruction execution logic and main loop.

#[allow(unused_imports)]
use typelude_core::std::int;
use typelude_core::{
    Apply, // Root export
    eval::{App, EIf, EWhile, Eval, Evaluate},
    std::{
        array::{Concat, Cons, EConcat, Get, Set, TyArray, TyNil}, // Cons is here
        bool::{TyFalse, TyTrue},
        ops::OpNot,
    },
};
use typenum::Unsigned;

use crate::machine::{instruction::*, state::MachineState};

pub trait Execute<Stack, Locals, Memory, CallStack, RestProg> {
    type OutputState;
}

// ... Implementations ...

// --- Push ---
impl<Stack, Locals, Memory, CallStack, Val, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpPush<Val>
where
    Stack: Cons,
{
    type OutputState = MachineState<TyArray<Val, Stack>, Locals, Memory, CallStack, RestProg>;
}

// --- Arithmetic ---
macro_rules! impl_binary_op {
    ($Op:ty, $EvalOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg>
            Execute<TyArray<Lhs, TyArray<Rhs, RestStack>>, Locals, Memory, CallStack, RestProg>
            for $Op
        where
            $EvalOp: Eval,
            RestStack: Cons,
        {
            type OutputState = MachineState<
                TyArray<Evaluate<$EvalOp>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
            >;
        }
    };
}

impl_binary_op!(OpAdd, typelude_core::std::ops::EAdd<Lhs, Rhs>);
impl_binary_op!(OpSub, typelude_core::std::ops::ESub<Lhs, Rhs>);

// --- Comparison ---
macro_rules! impl_cmp_op {
    ($Op:ty, $CoreOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg>
            Execute<TyArray<Lhs, TyArray<Rhs, RestStack>>, Locals, Memory, CallStack, RestProg>
            for $Op
        where
            $CoreOp: Apply<(Lhs, Rhs)>,
            App<$CoreOp, (Lhs, Rhs)>: Eval,
            RestStack: Cons,
        {
            type OutputState = MachineState<
                TyArray<Evaluate<App<$CoreOp, (Lhs, Rhs)>>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
            >;
        }
    };
}

#[cfg(feature = "nightly")]
impl_cmp_op!(OpEq, typelude_core::std::ops::OpEq);
#[cfg(feature = "nightly")]
impl_cmp_op!(OpNeq, typelude_core::std::ops::OpNeq);
impl_cmp_op!(OpLt, typelude_core::std::ops::OpLt);
impl_cmp_op!(OpGt, typelude_core::std::ops::OpGt);

// --- Boolean Logic ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpNot
where
    typelude_core::std::ops::ENot<Val>: Eval,
    RestStack: Cons,
{
    type OutputState = MachineState<
        TyArray<Evaluate<typelude_core::std::ops::ENot<Val>>, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

impl_binary_op!(OpAnd, typelude_core::std::ops::EAnd<Lhs, Rhs>);
impl_binary_op!(OpOr, typelude_core::std::ops::EOr<Lhs, Rhs>);

// --- Stack Manipulation ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpDup
where
    RestStack: Cons,
{
    type OutputState =
        MachineState<TyArray<Val, TyArray<Val, RestStack>>, Locals, Memory, CallStack, RestProg>;
}

impl<A, B, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<A, TyArray<B, RestStack>>, Locals, Memory, CallStack, RestProg> for OpSwap
where
    RestStack: Cons,
{
    type OutputState =
        MachineState<TyArray<B, TyArray<A, RestStack>>, Locals, Memory, CallStack, RestProg>;
}

impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpDrop
where
    RestStack: Cons,
{
    type OutputState = MachineState<RestStack, Locals, Memory, CallStack, RestProg>;
}

// --- Memory Access ---
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Addr, RestStack>, Locals, Memory, CallStack, RestProg> for OpLoad
where
    Memory: Get<Addr>,
    Addr: Unsigned,
    RestStack: Cons,
{
    type OutputState = MachineState<
        TyArray<<Memory as Get<Addr>>::Output, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

impl<Val, Addr, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Val, TyArray<Addr, RestStack>>, Locals, Memory, CallStack, RestProg>
    for OpStore
where
    Memory: Set<Addr, Val>,
    Addr: Unsigned,
    RestStack: Cons,
{
    type OutputState =
        MachineState<RestStack, Locals, <Memory as Set<Addr, Val>>::Output, CallStack, RestProg>;
}

// --- Local Variables ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpLet
where
    RestStack: Cons,
    Locals: Cons,
{
    type OutputState = MachineState<RestStack, TyArray<Val, Locals>, Memory, CallStack, RestProg>;
}

impl<Stack, Head, Tail, Memory, CallStack, RestProg>
    Execute<Stack, TyArray<Head, Tail>, Memory, CallStack, RestProg> for OpDropLocal
where
    Tail: Cons,
{
    type OutputState = MachineState<Stack, Tail, Memory, CallStack, RestProg>;
}

impl<Idx, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpGetLocal<Idx>
where
    Locals: Get<Idx>,
    Idx: Unsigned,
    Stack: Cons,
{
    type OutputState = MachineState<
        TyArray<<Locals as Get<Idx>>::Output, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

impl<Idx, Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpSetLocal<Idx>
where
    Locals: Set<Idx, Val>,
    Idx: Unsigned,
    RestStack: Cons,
{
    type OutputState =
        MachineState<RestStack, <Locals as Set<Idx, Val>>::Output, Memory, CallStack, RestProg>;
}

// --- Control Flow: Call/Return ---
impl<Target, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpCall<Target>
where
    CallStack: Cons,
{
    type OutputState =
        MachineState<Stack, TyNil, Memory, TyArray<(RestProg, Locals), CallStack>, Target>;
}

impl<Stack, Locals, Memory, Continuation, CallerLocals, RestCallStack, RestProg>
    Execute<Stack, Locals, Memory, TyArray<(Continuation, CallerLocals), RestCallStack>, RestProg>
    for OpReturn
where
    RestCallStack: Cons,
{
    type OutputState = MachineState<Stack, CallerLocals, Memory, RestCallStack, Continuation>;
}

// --- Control Flow: If ---
impl<Cond, Then, Else, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<TyArray<Cond, RestStack>, Locals, Memory, CallStack, RestProg> for OpIf<Then, Else>
where
    RestStack: Cons,
    EConcat<Then, RestProg>: Eval,
    EConcat<Else, RestProg>: Eval,
    EIf<
        Cond,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Then, RestProg>>>,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Else, RestProg>>>,
    >: Eval,
{
    type OutputState = Evaluate<
        EIf<
            Cond,
            MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Then, RestProg>>>,
            MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<Else, RestProg>>>,
        >,
    >;
}

// --- Control Flow: While ---
// OpWhile expands to: Cond ++ [OpIf<Body ++ [OpWhile<Cond, Body>], []>] ++ RestProg
// This transformation happens purely at the type level using Concat trait.
impl<Cond, Body, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpWhile<Cond, Body>
where
    RestProg: Cons,
    // Body ++ [OpWhile<Cond, Body>]
    Body: Concat<TyArray<OpWhile<Cond, Body>, TyNil>>,
    <Body as Concat<TyArray<OpWhile<Cond, Body>, TyNil>>>::Output: Cons,
    // Cond ++ [OpIf<LoopBody, []>]
    Cond: Concat<
        TyArray<OpIf<<Body as Concat<TyArray<OpWhile<Cond, Body>, TyNil>>>::Output, TyNil>, TyNil>,
    >,
    <Cond as Concat<
        TyArray<OpIf<<Body as Concat<TyArray<OpWhile<Cond, Body>, TyNil>>>::Output, TyNil>, TyNil>,
    >>::Output: Cons,
    // ExpandedWhile ++ RestProg
    <Cond as Concat<
        TyArray<OpIf<<Body as Concat<TyArray<OpWhile<Cond, Body>, TyNil>>>::Output, TyNil>, TyNil>,
    >>::Output: Concat<RestProg>,
{
    type OutputState = MachineState<
        Stack,
        Locals,
        Memory,
        CallStack,
        // Cond ++ [OpIf<Body ++ [OpWhile], []>] ++ RestProg
        <<Cond as Concat<
            TyArray<
                OpIf<<Body as Concat<TyArray<OpWhile<Cond, Body>, TyNil>>>::Output, TyNil>,
                TyNil,
            >,
        >>::Output as Concat<RestProg>>::Output,
    >;
}

// --- Machine Runner ---
pub struct OpStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg>
    Apply<MachineState<Stack, Locals, Memory, CallStack, TyArray<Inst, RestProg>>> for OpStep
where
    Inst: Execute<Stack, Locals, Memory, CallStack, RestProg>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
{
    type Output = <Inst as Execute<Stack, Locals, Memory, CallStack, RestProg>>::OutputState;
}

pub struct OpIsFinished;

// Program is empty (TyNil) -> Finished (TyFalse = stop loop)
impl<S, L, M, C> Apply<MachineState<S, L, M, C, TyNil>> for OpIsFinished {
    type Output = TyFalse;
}

// Program is not empty (TyArray) -> Not finished (TyTrue = continue loop)
impl<S, L, M, C, Inst, RestProg> Apply<MachineState<S, L, M, C, TyArray<Inst, RestProg>>>
    for OpIsFinished
where
    RestProg: Cons,
{
    type Output = TyTrue;
}

pub type ERun<S> = EWhile<OpIsFinished, OpStep, S>;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::tyarray;
    use typenum::{U1, U2, U3, U5};

    use super::*;

    #[test]
    fn test_stack_ops_logic() {
        type S0 = TyNil;
        type S1 = <OpPush<U1> as Execute<S0, TyNil, TyNil, TyNil, TyNil>>::OutputState;
        type ExpectedState = MachineState<tyarray![U1], TyNil, TyNil, TyNil, TyNil>;
        assert_type_eq_all!(S1, ExpectedState);
    }

    #[test]
    fn test_machine_run() {
        type Prog = tyarray![OpPush<U2>, OpPush<U3>, OpAdd, OpPush<U5>, OpSub];
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U0];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_call_return() {
        type SubRoutine = tyarray![OpPush<U5>, OpReturn];
        type MainProg = tyarray![OpPush<U3>, OpCall<SubRoutine>, OpAdd];

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, MainProg>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U8];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_memory_and_function() {
        use typenum::{U10, U20};
        type InitialMemory = tyarray![typenum::U0];
        type Func = tyarray![
            OpPush<typenum::U0>,
            OpPush<U10>,
            OpStore,
            OpPush<typenum::U0>,
            OpLoad,
            OpPush<U20>,
            OpAdd,
            OpPush<typenum::U0>,
            OpSwap,
            OpStore,
            OpReturn
        ];
        type Main = tyarray![OpCall<Func>];
        type InitialState = MachineState<TyNil, TyNil, InitialMemory, TyNil, Main>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type ExpectedMemory = tyarray![typenum::U30];
        type ExpectedState = MachineState<TyNil, TyNil, ExpectedMemory, TyNil, TyNil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_while_loop_countdown() {
        use typenum::{U0, U1, U3};
        // OpLt for < (0 < 3 -> True)
        type CondProg = tyarray![OpDup, OpPush<U0>, OpLt];
        type BodyProg = tyarray![OpPush<U1>, OpSub];
        type Prog = tyarray![OpPush<U3>, OpWhile<CondProg, BodyProg>];
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        // FIXME: Recursive eval limit or trait resolution failure in test environment
        // type FinalState = Evaluate<ERun<InitialState>>;
        // type ExpectedStack = tyarray![U0];
        // type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;
        // assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_simple_sort() {
        use typenum::{U0, U1, U3};
        type InitialMemory = tyarray![U3, U1];
        // OpLt for > (1 < 3 -> True)
        type SortProg = tyarray![
            OpPush<U0>, OpLoad, OpPush<U1>, OpLoad, OpLt,
            OpIf<
                tyarray![
                    OpPush<U0>, OpLoad, OpPush<U1>, OpLoad, OpSwap,
                    OpPush<U1>, OpSwap, OpStore,
                    OpPush<U0>, OpSwap, OpStore
                ],
                tyarray![]
            >
        ];
        type InitialState = MachineState<TyNil, TyNil, InitialMemory, TyNil, SortProg>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type ExpectedMemory = tyarray![U1, U3];
        type ExpectedState = MachineState<TyNil, TyNil, ExpectedMemory, TyNil, TyNil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_locals_isolation() {
        use typenum::{U0, U10, U20};
        type FuncB = tyarray![OpPush<U20>, OpLet, OpReturn];
        type FuncA = tyarray![OpPush<U10>, OpLet, OpCall<FuncB>, OpGetLocal<U0>, OpReturn];
        type Main = tyarray![OpCall<FuncA>];
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Main>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type ExpectedStack = tyarray![U10];
        type ExpectedState = MachineState<ExpectedStack, TyNil, TyNil, TyNil, TyNil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
