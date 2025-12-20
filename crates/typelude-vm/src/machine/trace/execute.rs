//! **Type-Level Stack Machine Traced Execution Logic**
//!
//! Implements machine instruction execution logic with history tracking.

use typenum::Unsigned;
use typelude_core::std::int::{OpAdd, OpSub};
use typelude_core::std::cmp::{OpEq, OpNeq, OpLt, OpGt};
use typelude_core::std::bool::{OpNot, OpAnd, OpOr};

use crate::machine::{instruction::*, trace::state::TracedMachineState};

use typelude_core::{
    eval::{App, EIf, ELit, EWhile, Eval, Evaluate},
    kernel::{traits::Apply, array::Cons},
    std::{
        array::{EConcat, Get, Set, TyArray, TyNil},
        // bool::OpNot,
    },
};

/// Trait to execute an instruction with history and return a new state
pub trait TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> {
    type OutputState;
}

// ... Implementations ...

// --- Simple Binary Ops ---
macro_rules! impl_traced_binary_op {
    ($Op:ty, $EvalOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg, History>
            TracedExecute<TyArray<Lhs, TyArray<Rhs, RestStack>>, Locals, Memory, CallStack, RestProg, History>
            for $Op
        where
            $EvalOp: Eval,
            RestStack: Cons,
            History: Cons,
        {
            type OutputState = TracedMachineState<
                TyArray<Evaluate<$EvalOp>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
                TyArray<$Op, History>
            >;
        }
    };
}

impl_traced_binary_op!(OpAdd, typelude_core::std::int::EAdd<Lhs, Rhs>);
impl_traced_binary_op!(OpSub, typelude_core::std::int::ESub<Lhs, Rhs>);
impl_traced_binary_op!(OpAnd, typelude_core::std::bool::EAnd<Lhs, Rhs>);
impl_traced_binary_op!(OpOr, typelude_core::std::bool::EOr<Lhs, Rhs>);

// --- Comparison Ops ---
macro_rules! impl_traced_cmp_op {
    ($Op:ty, $CoreOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg, History>
            TracedExecute<TyArray<Lhs, TyArray<Rhs, RestStack>>, Locals, Memory, CallStack, RestProg, History>
            for $Op
        where
            $CoreOp: Apply<(Lhs, Rhs)>,
            App<$CoreOp, (Lhs, Rhs)>: Eval,
            RestStack: Cons,
            History: Cons,
        {
            type OutputState = TracedMachineState<
                TyArray<Evaluate<App<$CoreOp, (Lhs, Rhs)>>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
                TyArray<$Op, History>
            >;
        }
    };
}

impl_traced_cmp_op!(OpEq, typelude_core::std::cmp::OpEq);
impl_traced_cmp_op!(OpNeq, typelude_core::std::cmp::OpNeq);
impl_traced_cmp_op!(OpLt, typelude_core::std::cmp::OpLt);
impl_traced_cmp_op!(OpGt, typelude_core::std::cmp::OpGt);

// --- Unary Ops ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpNot
where
    typelude_core::std::bool::ENot<Val>: Eval,
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        TyArray<Evaluate<typelude_core::std::bool::ENot<Val>>, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpNot, History>,
    >;
}

// --- Stack Ops ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpDup
where
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        TyArray<Val, TyArray<Val, RestStack>>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpDup, History>,
    >;
}

impl<A, B, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<A, TyArray<B, RestStack>>, Locals, Memory, CallStack, RestProg, History> for OpSwap
where
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        TyArray<B, TyArray<A, RestStack>>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpSwap, History>,
    >;
}

impl<Val, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Val, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpDrop
where
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        RestStack,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpDrop, History>,
    >;
}

// --- Push ---
impl<Val, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpPush<Val>
where
    Stack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        TyArray<Val, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpPush<Val>, History>,
    >;
}

// --- Implementation for Control Flow Ops ---

// OpLoad
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Addr, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpLoad
where
    Memory: Get<Addr>,
    Addr: Unsigned,
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        TyArray<<Memory as Get<Addr>>::Output, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpLoad, History>,
    >;
}

// OpStore
impl<Value, Addr, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<
        TyArray<Value, TyArray<Addr, RestStack>>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        History,
    > for OpStore
where
    Memory: Set<Addr, Value>,
    Addr: Unsigned,
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        RestStack,
        Locals,
        <Memory as Set<Addr, Value>>::Output,
        CallStack,
        RestProg,
        TyArray<OpStore, History>,
    >;
}

// OpGetLocal
impl<Index, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpGetLocal<Index>
where
    Locals: Get<Index>,
    Index: Unsigned,
    Stack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        TyArray<<Locals as Get<Index>>::Output, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpGetLocal<Index>, History>,
    >;
}

// OpSetLocal
impl<Index, Value, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Value, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpSetLocal<Index>
where
    Locals: Set<Index, Value>,
    Index: Unsigned,
    RestStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        RestStack,
        <Locals as Set<Index, Value>>::Output,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpSetLocal<Index>, History>,
    >;
}

// OpLet
impl<Value, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Value, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpLet
where
    RestStack: Cons,
    Locals: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        RestStack,
        TyArray<Value, Locals>,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpLet, History>,
    >;
}

// OpDropLocal
impl<Stack, Head, Tail, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, TyArray<Head, Tail>, Memory, CallStack, RestProg, History> for OpDropLocal
where
    Tail: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        Stack,
        Tail,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpDropLocal, History>,
    >;
}

// OpCall
impl<TargetProg, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpCall<TargetProg>
where
    CallStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        Stack,
        TyNil,
        Memory,
        TyArray<(RestProg, Locals), CallStack>,
        TargetProg,
        TyArray<OpCall<TargetProg>, History>,
    >;
}

// OpReturn
impl<Stack, Locals, Memory, Continuation, CallerLocals, RestCallStack, RestProg, History>
    TracedExecute<
        Stack,
        Locals,
        Memory,
        TyArray<(Continuation, CallerLocals), RestCallStack>,
        RestProg,
        History,
    > for OpReturn
where
    RestCallStack: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        Stack,
        CallerLocals,
        Memory,
        RestCallStack,
        Continuation,
        TyArray<OpReturn, History>,
    >;
}

// OpWhile
impl<CondProg, BodyProg, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>
    for OpWhile<CondProg, BodyProg>
where
    History: Cons,
    EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>: Eval,
    EConcat<
        CondProg,
        TyArray<
            OpIf<Evaluate<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>, TyNil>,
            TyNil,
        >,
    >: Eval,
    EConcat<
        Evaluate<
            EConcat<
                CondProg,
                TyArray<
                    OpIf<
                        Evaluate<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>,
                        TyNil,
                    >,
                    TyNil,
                >,
            >,
        >,
        RestProg
    >: Eval,
{
    type OutputState = TracedMachineState<
        Stack,
        Locals,
        Memory,
        CallStack,
        Evaluate<
            EConcat<
                Evaluate<
                    EConcat<
                        CondProg,
                        TyArray<
                            OpIf<
                                Evaluate<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>,
                                TyNil,
                            >,
                            TyNil,
                        >,
                    >,
                >,
                RestProg
            >
        >,
        TyArray<OpWhile<CondProg, BodyProg>, History>,
    >;
}

// OpIf
impl<Cond, RestStack, Locals, Memory, CallStack, Then, Else, RestProg, History>
    TracedExecute<TyArray<Cond, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpIf<Then, Else>
where
    RestStack: Cons,
    History: Cons,
    EConcat<Then, RestProg>: Eval,
    EConcat<Else, RestProg>: Eval,
    EIf<
        Cond,
        TracedMachineState<
            RestStack,
            Locals,
            Memory,
            CallStack,
            Evaluate<EConcat<Then, RestProg>>,
            TyArray<OpIf<Then, Else>, History>,
        >,
        TracedMachineState<
            RestStack,
            Locals,
            Memory,
            CallStack,
            Evaluate<EConcat<Else, RestProg>>,
            TyArray<OpIf<Then, Else>, History>,
        >,
    >: Eval,
{
    type OutputState = Evaluate<
        EIf<
            Cond,
            TracedMachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluate<EConcat<Then, RestProg>>,
                TyArray<OpIf<Then, Else>, History>,
            >,
            TracedMachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluate<EConcat<Else, RestProg>>,
                TyArray<OpIf<Then, Else>, History>,
            >,
        >,
    >;
}

/// Step function for Traced Machine
pub struct OpTracedStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg, History>
    Apply<TracedMachineState<Stack, Locals, Memory, CallStack, TyArray<Inst, RestProg>, History>>
    for OpTracedStep
where
    Inst: TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
    History: Cons,
{
    type Output = ELit<
        <Inst as TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>>::OutputState,
    >;
}

pub struct OpTracedIsFinished;

impl<S, L, M, C, P, H> Apply<TracedMachineState<S, L, M, C, P, H>> for OpTracedIsFinished
where
    typelude_core::std::array::EIsEmpty<ELit<P>>: Eval,
    App<typelude_core::std::bool::OpNot, typelude_core::std::array::EIsEmpty<ELit<P>>>: Eval,
{
    type Output = App<typelude_core::std::bool::OpNot, typelude_core::std::array::EIsEmpty<ELit<P>>>;
}

/// Runner for traced execution
pub type ETracedRun<S> = EWhile<OpTracedIsFinished, OpTracedStep, S>;
