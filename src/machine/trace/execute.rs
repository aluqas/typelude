use typenum::Unsigned;

use crate::{
    eval::{EApply, EIf, EWhile, Evaluable, Evaluator},
    machine::{execution::RunStep, instruction::*, trace::TracedMachineState},
    std::{
        array::{Cons, EConcat, FIsEmpty, Get, Set, TyArray, TyNil},
        bool::FNot,
        traits::EFunction,
    },
};

/// Similar to Execute, but maintains a History log.
pub trait TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> {
    type OutputState;
}

// --- Implementation for Stack-only Ops (via RunStep) ---

macro_rules! impl_traced_execute_via_runstep {
    ($Inst:ty) => {
        impl<Stack, Locals, Memory, CallStack, RestProg, History> TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for $Inst
        where
            Self: RunStep<Stack>,
            RestProg: Cons,
            History: Cons,
        {
            type OutputState = TracedMachineState<
                <Self as RunStep<Stack>>::OutputStack,
                Locals,
                Memory,
                CallStack,
                RestProg,
                TyArray<$Inst, History> // Prepend instruction to history
            >;
        }
    };
    ($Inst:ident < $($T:ident),+ >) => {
        impl<$($T),+, Stack, Locals, Memory, CallStack, RestProg, History> TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for $Inst<$($T),+>
        where
            Self: RunStep<Stack>,
            RestProg: Cons,
            History: Cons,
        {
            type OutputState = TracedMachineState<
                <Self as RunStep<Stack>>::OutputStack,
                Locals,
                Memory,
                CallStack,
                RestProg,
                TyArray<$Inst<$($T),+>, History>
            >;
        }
    };
}

impl<Val, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for OpPush<Val>
where
    Self: RunStep<Stack>,
    RestProg: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        <Self as RunStep<Stack>>::OutputStack,
        Locals,
        Memory,
        CallStack,
        RestProg,
        TyArray<OpPush<Val>, History>,
    >;
}

impl_traced_execute_via_runstep!(OpAdd);
impl_traced_execute_via_runstep!(OpSub);
impl_traced_execute_via_runstep!(OpDup);
impl_traced_execute_via_runstep!(OpSwap);
impl_traced_execute_via_runstep!(OpDrop);
impl_traced_execute_via_runstep!(OpEq);
impl_traced_execute_via_runstep!(OpNeq);
impl_traced_execute_via_runstep!(OpLt);
impl_traced_execute_via_runstep!(OpGt);
impl_traced_execute_via_runstep!(OpNot);
impl_traced_execute_via_runstep!(OpAnd);
impl_traced_execute_via_runstep!(OpOr);

// --- Implementation for Control Flow Ops ---

// OpLoad
impl<Addr, RestStack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<TyArray<Addr, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpLoad
where
    Addr: Unsigned,
    TyArray<Addr, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Get<Addr>,
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
    Addr: Unsigned,
    TyArray<Value, TyArray<Addr, RestStack>>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    Memory: Set<Addr, Value>,
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
    Index: Unsigned,
    Stack: Cons,
    Locals: Get<Index>,
    RestProg: Cons,
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
    Index: Unsigned,
    TyArray<Value, RestStack>: Cons,
    RestStack: Cons,
    Locals: Set<Index, Value>,
    RestProg: Cons,
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
    TyArray<Value, RestStack>: Cons,
    RestStack: Cons,
    Locals: Cons,
    RestProg: Cons,
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
    Stack: Cons,
    TyArray<Head, Tail>: Cons,
    Tail: Cons,
    RestProg: Cons,
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
    Stack: Cons,
    Locals: Cons,
    Memory: Cons,
    CallStack: Cons,
    TargetProg: Cons,
    RestProg: Cons,
    History: Cons,
{
    type OutputState = TracedMachineState<
        Stack,
        TyNil,
        Memory,
        TyArray<TyArray<RestProg, Locals>, CallStack>,
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
        TyArray<TyArray<Continuation, CallerLocals>, RestCallStack>,
        RestProg,
        History,
    > for OpReturn
where
    Stack: Cons,
    Memory: Cons,
    TyArray<TyArray<Continuation, CallerLocals>, RestCallStack>: Cons,
    TyArray<Continuation, CallerLocals>: Cons,
    Continuation: Cons,
    CallerLocals: Cons,
    RestCallStack: Cons,
    RestProg: Cons,
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
    Stack: Cons,
    Locals: Cons,
    CondProg: Cons,
    BodyProg: Cons,
    RestProg: Cons,
    History: Cons,
    EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>: Evaluable,
    EConcat<
        CondProg,
        TyArray<
            OpIf<Evaluator<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>, TyNil>,
            RestProg,
        >,
    >: Evaluable,
{
    type OutputState = TracedMachineState<
        Stack,
        Locals,
        Memory,
        CallStack,
        Evaluator<
            EConcat<
                CondProg,
                TyArray<
                    OpIf<
                        Evaluator<EConcat<BodyProg, TyArray<OpWhile<CondProg, BodyProg>, TyNil>>>,
                        TyNil,
                    >,
                    RestProg,
                >,
            >,
        >,
        TyArray<OpWhile<CondProg, BodyProg>, History>,
    >;
}

// OpIf
impl<Cond, RestStack, Locals, Memory, CallStack, Then, Else, RestProg, History>
    TracedExecute<TyArray<Cond, RestStack>, Locals, Memory, CallStack, RestProg, History>
    for OpIf<Then, Else>
where
    TyArray<Cond, RestStack>: Cons,
    RestStack: Cons,
    RestProg: Cons,
    History: Cons,
    Then: Cons,
    Else: Cons,
    EConcat<Then, RestProg>: Evaluable,
    EConcat<Else, RestProg>: Evaluable,
    EIf<
        Cond,
        TracedMachineState<
            RestStack,
            Locals,
            Memory,
            CallStack,
            Evaluator<EConcat<Then, RestProg>>,
            TyArray<OpIf<Then, Else>, History>,
        >,
        TracedMachineState<
            RestStack,
            Locals,
            Memory,
            CallStack,
            Evaluator<EConcat<Else, RestProg>>,
            TyArray<OpIf<Then, Else>, History>,
        >,
    >: Evaluable,
{
    type OutputState = Evaluator<
        EIf<
            Cond,
            TracedMachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluator<EConcat<Then, RestProg>>,
                TyArray<OpIf<Then, Else>, History>,
            >,
            TracedMachineState<
                RestStack,
                Locals,
                Memory,
                CallStack,
                Evaluator<EConcat<Else, RestProg>>,
                TyArray<OpIf<Then, Else>, History>,
            >,
        >,
    >;
}

/// Step function for Traced Machine
pub struct FTracedStep;

impl<Stack, Locals, Memory, CallStack, Inst, RestProg, History>
    EFunction<
        TracedMachineState<Stack, Locals, Memory, CallStack, TyArray<Inst, RestProg>, History>,
    > for FTracedStep
where
    Inst: TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>,
    TyArray<Inst, RestProg>: Cons,
    RestProg: Cons,
    History: Cons,
{
    type Output =
        <Inst as TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History>>::OutputState;
}

impl<S, L, M, C, P, H> Evaluable for EApply<FTracedStep, TracedMachineState<S, L, M, C, P, H>>
where
    FTracedStep: EFunction<TracedMachineState<S, L, M, C, P, H>>,
{
    type Output = <FTracedStep as EFunction<TracedMachineState<S, L, M, C, P, H>>>::Output;
}

pub struct FTracedIsFinished;

impl<S, L, M, C, P, H> EFunction<TracedMachineState<S, L, M, C, P, H>> for FTracedIsFinished
where
    EApply<FIsEmpty, P>: Evaluable,
    Evaluator<EApply<FIsEmpty, P>>: crate::std::bool::NotHelper,
{
    type Output = Evaluator<EApply<FNot, EApply<FIsEmpty, P>>>;
}

/// Runner for traced execution
pub type ETracedRun<S> = EWhile<FTracedIsFinished, FTracedStep, S>;
