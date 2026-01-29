//! **Type-Level Stack Machine Traced Execution Logic**
//!
//! Implements machine instruction execution logic with history tracking.

use typelude_std::core::{Apply, ELit, Eval, Evaluate};
#[cfg(feature = "nightly")]
use typelude_std::std::ops::{OpEq, OpNeq};
use typelude_std::{
    expr::{EIf, EWhile},
    std::{
        array::{Array, EConcat, Get, IsList, Nil, Set},
        ops::{
            EAdd,
            EAnd,
            ENot,
            EOr,
            ESub, // Expressions
        },
    },
};
use typenum::Unsigned;

use crate::machine::{instruction::*, trace::state::TracedMachineState};

/// Trait to execute an instruction with history and return a new state
pub trait TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> {
    type OutputState;
}

macro_rules! impl_traced_binary_op {
    ($Op:ty, $EvalOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg, History>
            TracedExecute<
                Array<Lhs, Array<Rhs, RestStack>>,
                Locals,
                Memory,
                CallStack,
                RestProg,
                History,
            > for $Op
        where
            $EvalOp: Eval,
            RestStack: IsList,
            History: IsList,
        {
            type OutputState = TracedMachineState<
                Array<Evaluate<$EvalOp>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
                Array<$Op, History>,
            >;
        }
    };
}

impl_traced_binary_op!(OpAdd, EAdd<Lhs, Rhs>);
impl_traced_binary_op!(OpSub, ESub<Lhs, Rhs>);
impl_traced_binary_op!(OpAnd, EAnd<Lhs, Rhs>);
impl_traced_binary_op!(OpOr, EOr<Lhs, Rhs>);

macro_rules! impl_traced_cmp_op {
    ($Op:ty, $EvalOp:ty) => {
        impl<Lhs, Rhs, RestStack, Locals, Memory, CallStack, RestProg, History>
            TracedExecute<
                Array<Lhs, Array<Rhs, RestStack>>,
                Locals,
                Memory,
                CallStack,
                RestProg,
                History,
            > for $Op
        where
            $EvalOp: Eval,
            RestStack: IsList,
            History: IsList,
        {
            type OutputState = TracedMachineState<
                Array<Evaluate<$EvalOp>, RestStack>,
                Locals,
                Memory,
                CallStack,
                RestProg,
                Array<$Op, History>,
            >;
        }
    };
}

#[cfg(feature = "nightly")]
impl_traced_cmp_op!(OpEq, typelude_std::std::ops::EEq<Lhs, Rhs>);
#[cfg(feature = "nightly")]
impl_traced_cmp_op!(OpNeq, typelude_std::std::ops::ENeq<Lhs, Rhs>);
impl_traced_cmp_op!(OpLt, typelude_std::std::ops::ELt<Lhs, Rhs>);
impl_traced_cmp_op!(OpGt, typelude_std::std::ops::EGt<Lhs, Rhs>);

// --- Unary Ops ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<Val, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpNot
where
    ENot<Val>: Eval,
    RestStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Array<Evaluate<ENot<Val>>, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        Array<OpNot, History>,
    >;
}

// --- Stack Ops ---
impl<Val, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<Val, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpDup
where
    RestStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Array<Val, Array<Val, RestStack>>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        Array<OpDup, History>,
    >;
}

impl<A, B, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<A, Array<B, RestStack>>, Locals, Memory, CallStack, RestProg, History>
    for OpSwap
where
    RestStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Array<B, Array<A, RestStack>>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        Array<OpSwap, History>,
    >;
}

impl<Val, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<Val, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpDrop
where
    RestStack: IsList,
    History: IsList,
{
    type OutputState =
        TracedMachineState<RestStack, Locals, Memory, CallStack, RestProg, Array<OpDrop, History>>;
}

// --- Push ---
impl<Val, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpPush<Val>
where
    Stack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Array<Val, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        Array<OpPush<Val>, History>,
    >;
}

// --- Implementation for Control Flow Ops ---

// OpLoad
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<Addr, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpLoad
where
    Memory: Get<Addr>,
    Addr: Unsigned,
    RestStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Array<<Memory as Get<Addr>>::Output, RestStack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        Array<OpLoad, History>,
    >;
}

// OpStore
impl<Value, Addr, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<
        Array<Value, Array<Addr, RestStack>>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        History,
    > for OpStore
where
    Memory: Set<Addr, Value>,
    Addr: Unsigned,
    RestStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        RestStack,
        Locals,
        <Memory as Set<Addr, Value>>::Output,
        CallStack,
        RestProg,
        Array<OpStore, History>,
    >;
}

// OpGetLocal
impl<Index, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpGetLocal<Index>
where
    Locals: Get<Index>,
    Index: Unsigned,
    Stack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Array<<Locals as Get<Index>>::Output, Stack>,
        Locals,
        Memory,
        CallStack,
        RestProg,
        Array<OpGetLocal<Index>, History>,
    >;
}

// OpSetLocal
impl<Index, Value, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<Value, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpSetLocal<Index>
where
    Locals: Set<Index, Value>,
    Index: Unsigned,
    RestStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        RestStack,
        <Locals as Set<Index, Value>>::Output,
        Memory,
        CallStack,
        RestProg,
        Array<OpSetLocal<Index>, History>,
    >;
}

// OpLet
impl<Value, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Array<Value, RestStack>, Locals, Memory, CallStack, RestProg, History> for OpLet
where
    RestStack: IsList,
    Locals: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        RestStack,
        Array<Value, Locals>,
        Memory,
        CallStack,
        RestProg,
        Array<OpLet, History>,
    >;
}

// OpDropLocal
impl<Stack, Head, Tail, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Array<Head, Tail>, Memory, CallStack, RestProg, History> for OpDropLocal
where
    Tail: IsList,
    History: IsList,
{
    type OutputState =
        TracedMachineState<Stack, Tail, Memory, CallStack, RestProg, Array<OpDropLocal, History>>;
}

// OpCall
impl<TargetProg, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpCall<TargetProg>
where
    CallStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Stack,
        Nil,
        Memory,
        Array<(RestProg, Locals), CallStack>,
        TargetProg,
        Array<OpCall<TargetProg>, History>,
    >;
}

// OpReturn
impl<Stack, Locals, Memory, Continuation, CallerLocals, RestCallStack, RestProg, History>
    TracedExecute<
        Stack,
        Locals,
        Memory,
        Array<(Continuation, CallerLocals), RestCallStack>,
        RestProg,
        History,
    > for OpReturn
where
    RestCallStack: IsList,
    History: IsList,
{
    type OutputState = TracedMachineState<
        Stack,
        CallerLocals,
        Memory,
        RestCallStack,
        Continuation,
        Array<OpReturn, History>,
    >;
}

// OpWhile
impl<CondProg, BodyProg, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>
    for OpWhile<CondProg, BodyProg>
where
    History: IsList,
    EConcat<BodyProg, Array<OpWhile<CondProg, BodyProg>, Nil>>: Eval,
    EConcat<
        CondProg,
        Array<
            OpIf<Evaluate<EConcat<BodyProg, Array<OpWhile<CondProg, BodyProg>, Nil>>>, Nil>,
            Nil,
        >,
    >: Eval,
    EConcat<
        Evaluate<
            EConcat<
                CondProg,
                Array<
                    OpIf<
                        Evaluate<EConcat<BodyProg, Array<OpWhile<CondProg, BodyProg>, Nil>>>,
                        Nil,
                    >,
                    Nil,
                >,
            >,
        >,
        RestProg,
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
                        Array<
                            OpIf<
                                Evaluate<
                                    EConcat<BodyProg, Array<OpWhile<CondProg, BodyProg>, Nil>>,
                                >,
                                Nil,
                            >,
                            Nil,
                        >,
                    >,
                >,
                RestProg,
            >,
        >,
        Array<OpWhile<CondProg, BodyProg>, History>,
    >;
}

// OpIf
impl<Cond, RestStack, Locals, Memory, CallStack, Then, Else, RestProg, History>
    TracedExecute<Array<Cond, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpIf<Then, Else>
where
    RestStack: IsList,
    History: IsList,
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
            Array<OpIf<Then, Else>, History>,
        >,
        TracedMachineState<
            RestStack,
            Locals,
            Memory,
            CallStack,
            Evaluate<EConcat<Else, RestProg>>,
            Array<OpIf<Then, Else>, History>,
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
                Array<OpIf<Then, Else>, History>,
            >,
            TracedMachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluate<EConcat<Else, RestProg>>,
                Array<OpIf<Then, Else>, History>,
            >,
        >,
    >;
}

/// Step function for Traced Machine
pub struct OpTracedStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg, History>
    Apply<TracedMachineState<Stack, Locals, Memory, CallStack, Array<Inst, RestProg>, History>>
    for OpTracedStep
where
    Inst: TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>,
    Array<Inst, RestProg>: IsList,
    RestProg: IsList,
    History: IsList,
{
    type Output = ELit<
        <Inst as TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>>::OutputState,
    >;
}

pub struct OpTracedIsFinished;

impl<S, L, M, C, P, H> Apply<TracedMachineState<S, L, M, C, P, H>> for OpTracedIsFinished
where
    typelude_std::std::array::EIsEmpty<ELit<P>>: Eval,
    typelude_std::std::ops::ENot<typelude_std::std::array::EIsEmpty<ELit<P>>>: Eval,
{
    type Output =
        Evaluate<typelude_std::std::ops::ENot<typelude_std::std::array::EIsEmpty<ELit<P>>>>;
}

/// Runner for traced execution
pub type ETracedRun<S> = EWhile<OpTracedIsFinished, OpTracedStep, S>;
