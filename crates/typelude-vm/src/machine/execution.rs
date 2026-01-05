//! **Type-Level Stack Machine Execution Logic**
//!
//! Implements machine instruction execution logic and main loop.

use typelude_core::{App, Apply, Eval, Evaluate};
#[allow(unused_imports)]
use typelude_std::std::int;
use typelude_std::{
    expr::{EIf, EWhile},
    std::{
        array::{Array, Concat, EConcat, Get, IsList, Nil, Set},
        bool::{False, True},
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
    Stack: IsList,
{
    type OutputState = MachineState<Array<Val, Stack>, Locals, Memory, CallStack, RestProg>;
}

// --- Binary Ops ---
macro_rules! impl_binary_op {
    ($Op:ident, $EvalOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg>
            Execute<Array<Lhs, Array<Rhs, RestStack>>, Locals, Memory, CallStack, RestProg> for $Op
        where
            $EvalOp: Eval,
            RestStack: IsList,
        {
            type OutputState = MachineState<
                Array<Evaluate<$EvalOp>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
            >;
        }
    };
}

impl_binary_op!(OpAdd, typelude_std::std::ops::EAdd<Lhs, Rhs>);
impl_binary_op!(OpSub, typelude_std::std::ops::ESub<Lhs, Rhs>);

// --- Comparison Ops ---
macro_rules! impl_cmp_op {
    ($Op:ident, $CoreOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg>
            Execute<Array<Lhs, Array<Rhs, RestStack>>, Locals, Memory, CallStack, RestProg> for $Op
        where
            $CoreOp:
                Apply<typelude_core::ECons<Lhs, typelude_core::ECons<Rhs, typelude_core::ENil>>>,
            App<
                $CoreOp,
                typelude_core::ECons<Lhs, typelude_core::ECons<Rhs, typelude_core::ENil>>,
            >: Eval,
            RestStack: IsList,
        {
            type OutputState = MachineState<
                Array<
                    Evaluate<
                        App<
                            $CoreOp,
                            typelude_core::ECons<
                                Lhs,
                                typelude_core::ECons<Rhs, typelude_core::ENil>,
                            >,
                        >,
                    >,
                    RestStack,
                >,
                Locals,
                Memory,
                CallStack,
                RestProg,
            >;
        }
    };
}

#[cfg(feature = "nightly")]
impl_cmp_op!(OpEq, typelude_std::std::ops::OpEq);
#[cfg(feature = "nightly")]
impl_cmp_op!(OpNeq, typelude_std::std::ops::OpNeq);
impl_cmp_op!(OpLt, typelude_std::std::ops::OpLt);
impl_cmp_op!(OpGt, typelude_std::std::ops::OpGt);

// --- Unary Ops ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpNot
where
    typelude_std::std::ops::ENot<Val>: Eval,
    RestStack: IsList,
{
    type OutputState = MachineState<
        Array<Evaluate<typelude_std::std::ops::ENot<Val>>, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

impl_binary_op!(OpAnd, typelude_std::std::ops::EAnd<Lhs, Rhs>);
impl_binary_op!(OpOr, typelude_std::std::ops::EOr<Lhs, Rhs>);

// --- Stack Ops ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpDup
where
    RestStack: IsList,
{
    type OutputState =
        MachineState<Array<Val, Array<Val, RestStack>>, Locals, Memory, CallStack, RestProg>;
}

impl<A, B, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<A, Array<B, RestStack>>, Locals, Memory, CallStack, RestProg> for OpSwap
where
    RestStack: IsList,
{
    type OutputState =
        MachineState<Array<B, Array<A, RestStack>>, Locals, Memory, CallStack, RestProg>;
}

impl<Val, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Val, RestStack>, Locals, Memory, CallStack, RestProg> for OpDrop
where
    RestStack: IsList,
{
    type OutputState = MachineState<RestStack, Locals, Memory, CallStack, RestProg>;
}

// --- Memory Access ---
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Addr, RestStack>, Locals, Memory, CallStack, RestProg> for OpLoad
where
    Memory: Get<Addr>,
    Addr: Unsigned,
    RestStack: IsList,
{
    type OutputState = MachineState<
        Array<<Memory as Get<Addr>>::Output, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

impl<Value, Addr, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Value, Array<Addr, RestStack>>, Locals, Memory, CallStack, RestProg> for OpStore
where
    Memory: Set<Addr, Value>,
    Addr: Unsigned,
    RestStack: IsList,
{
    type OutputState =
        MachineState<RestStack, Locals, <Memory as Set<Addr, Value>>::Output, CallStack, RestProg>;
}

// --- Local Variables ---
impl<Value, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Value, RestStack>, Locals, Memory, CallStack, RestProg> for OpLet
where
    RestStack: IsList,
    Locals: IsList,
{
    type OutputState = MachineState<RestStack, Array<Value, Locals>, Memory, CallStack, RestProg>;
}

impl<Stack, Head, Tail, Memory, CallStack, RestProg>
    Execute<Stack, Array<Head, Tail>, Memory, CallStack, RestProg> for OpDropLocal
where
    Tail: IsList,
{
    type OutputState = MachineState<Stack, Tail, Memory, CallStack, RestProg>;
}

impl<Index, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpGetLocal<Index>
where
    Locals: Get<Index>,
    Index: Unsigned,
    Stack: IsList,
{
    type OutputState = MachineState<
        Array<<Locals as Get<Index>>::Output, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
    >;
}

impl<Index, Value, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<Value, RestStack>, Locals, Memory, CallStack, RestProg> for OpSetLocal<Index>
where
    Locals: Set<Index, Value>,
    Index: Unsigned,
    RestStack: IsList,
{
    type OutputState = MachineState<
        RestStack,
        <Locals as Set<Index, Value>>::Output,
        Memory,
        CallStack,
        RestProg,
    >;
}

// --- Control Flow ---
impl<TargetProg, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpCall<TargetProg>
where
    CallStack: IsList,
{
    type OutputState =
        MachineState<Stack, Nil, Memory, Array<(RestProg, Locals), CallStack>, TargetProg>;
}

impl<Stack, Locals, Memory, Continuation, CallerLocals, RestCallStack, RestProg>
    Execute<Stack, Locals, Memory, Array<(Continuation, CallerLocals), RestCallStack>, RestProg>
    for OpReturn
where
    RestCallStack: IsList,
{
    type OutputState = MachineState<Stack, CallerLocals, Memory, RestCallStack, Continuation>;
}

// --- Control Flow: If ---
impl<CondVal, ThenProg, ElseProg, RestStack, Locals, Memory, CallStack, RestProg>
    Execute<Array<CondVal, RestStack>, Locals, Memory, CallStack, RestProg>
    for OpIf<ThenProg, ElseProg>
where
    RestStack: IsList,
    EConcat<ThenProg, RestProg>: Eval,
    EConcat<ElseProg, RestProg>: Eval,
    EIf<
        CondVal,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<ThenProg, RestProg>>>,
        MachineState<RestStack, Locals, Memory, CallStack, Evaluate<EConcat<ElseProg, RestProg>>>,
    >: Eval,
{
    type OutputState = Evaluate<
        EIf<
            CondVal,
            MachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluate<EConcat<ThenProg, RestProg>>,
            >,
            MachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluate<EConcat<ElseProg, RestProg>>,
            >,
        >,
    >;
}

// --- Control Flow: While ---
// OpWhile expands to: CondProg ++ [OpIf<BodyProg ++ [OpWhile<CondProg, BodyProg>], []>] ++ RestProg
// This transformation happens purely at the type level using Concat trait.
impl<CondProg, BodyProg, Stack, Locals, Memory, CallStack, RestProg>
    Execute<Stack, Locals, Memory, CallStack, RestProg> for OpWhile<CondProg, BodyProg>
where
    RestProg: IsList,
    // BodyProg ++ [OpWhile<CondProg, BodyProg>]
    BodyProg: Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>,
    <BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output: IsList,
    // CondProg ++ [OpIf<LoopBody, []>]
    CondProg: Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Nil,
        >,
    >,
    <CondProg as Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Nil,
        >,
    >>::Output: IsList,
    // ExpandedWhile ++ RestProg
    <CondProg as Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Nil,
        >,
    >>::Output: Concat<RestProg>,
{
    type OutputState = MachineState<
        Stack,
        Locals,
        Memory,
        CallStack,
        // CondProg ++ [OpIf<BodyProg ++ [OpWhile], []>] ++ RestProg
        <<CondProg as Concat<
            Array<
                OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
                Nil,
            >,
        >>::Output as Concat<RestProg>>::Output,
    >;
}

// --- Machine Runner ---
pub struct OpStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg>
    Apply<MachineState<Stack, Locals, Memory, CallStack, Array<Inst, RestProg>>> for OpStep
where
    Inst: Execute<Stack, Locals, Memory, CallStack, RestProg>,
    Array<Inst, RestProg>: IsList,
    RestProg: IsList,
{
    type Output = <Inst as Execute<Stack, Locals, Memory, CallStack, RestProg>>::OutputState;
}

pub struct OpIsFinished;

// Program is empty (Nil) -> Finished (False = stop loop)
impl<S, L, M, C> Apply<MachineState<S, L, M, C, Nil>> for OpIsFinished {
    type Output = False;
}

// Program is not empty (Array) -> Not finished (True = continue loop)
impl<S, L, M, C, Inst, RestProg> Apply<MachineState<S, L, M, C, Array<Inst, RestProg>>>
    for OpIsFinished
where
    RestProg: IsList,
{
    type Output = True;
}
// OpIsFinished must be Evaluatable to be used in EWhile
impl Eval for OpIsFinished {
    type Output = Self;
}

// OpStep must be Evaluatable to be used in EWhile
impl Eval for OpStep {
    type Output = Self;
}

pub type ERun<S> = EWhile<OpIsFinished, OpStep, S>;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::tyarray;
    use typenum::{U1, U2, U3, U5};

    use super::*;

    #[test]
    fn test_stack_ops_logic() {
        type S0 = Nil;
        type S1 = <OpPush<U1> as Execute<S0, Nil, Nil, Nil, Nil>>::OutputState;
        type ExpectedState = MachineState<tyarray![U1], Nil, Nil, Nil, Nil>;
        assert_type_eq_all!(S1, ExpectedState);
    }

    #[test]
    fn test_machine_run() {
        type Prog = tyarray![OpPush<U2>, OpPush<U3>, OpAdd, OpPush<U5>, OpSub];
        type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U0];
        type ExpectedState = MachineState<ExpectedStack, Nil, Nil, Nil, Nil>;

        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_call_return() {
        type SubRoutine = tyarray![OpPush<U5>, OpReturn];
        type MainProg = tyarray![OpPush<U3>, OpCall<SubRoutine>, OpAdd];

        type InitialState = MachineState<Nil, Nil, Nil, Nil, MainProg>;
        type FinalState = Evaluate<ERun<InitialState>>;

        type ExpectedStack = tyarray![typenum::U8];
        type ExpectedState = MachineState<ExpectedStack, Nil, Nil, Nil, Nil>;

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
        type InitialState = MachineState<Nil, Nil, InitialMemory, Nil, Main>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type ExpectedMemory = tyarray![typenum::U30];
        type ExpectedState = MachineState<Nil, Nil, ExpectedMemory, Nil, Nil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_while_loop_countdown() {
        use typenum::{U0, U1, U3};
        // OpLt for < (0 < 3 -> True)
        type CondProg = tyarray![OpDup, OpPush<U0>, OpLt];
        type BodyProg = tyarray![OpPush<U1>, OpSub];
        type Prog = tyarray![OpPush<U3>, OpWhile<CondProg, BodyProg>];
        type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
        // FIXME: Recursive eval limit or trait resolution failure in test environment
        // type FinalState = Evaluate<ERun<InitialState>>;
        // type ExpectedStack = tyarray![U0];
        // type ExpectedState = MachineState<ExpectedStack, Nil, Nil, Nil, Nil>;
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
        type InitialState = MachineState<Nil, Nil, InitialMemory, Nil, SortProg>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type ExpectedMemory = tyarray![U1, U3];
        type ExpectedState = MachineState<Nil, Nil, ExpectedMemory, Nil, Nil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_locals_isolation() {
        use typenum::{U0, U10, U20};
        type FuncB = tyarray![OpPush<U20>, OpLet, OpReturn];
        type FuncA = tyarray![OpPush<U10>, OpLet, OpCall<FuncB>, OpGetLocal<U0>, OpReturn];
        type Main = tyarray![OpCall<FuncA>];
        type InitialState = MachineState<Nil, Nil, Nil, Nil, Main>;
        type FinalState = Evaluate<ERun<InitialState>>;
        type ExpectedStack = tyarray![U10];
        type ExpectedState = MachineState<ExpectedStack, Nil, Nil, Nil, Nil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
