use core::marker::PhantomData;

use typelude_std::{
    core::{ELit, Eval, Evaluate, TyFn},
    std::col::array::{Array, Concat, IsList, Nil},
};

use crate::{
    composed::{
        core::{Bind, Monad},
        interpret::InterpInstr,
        vm::{
            effects::{InvalidCondition, PushTrace, PutVm, Then, ThrowVm, VmFx, VmTrace, VmTrap},
            state::VmState,
        },
    },
    machine::instr::core::{OpIf, OpWhile},
};

pub trait ToBranchBool {
    type Output;
}

impl ToBranchBool for typelude_std::std::prim::bool::True {
    type Output = typelude_std::std::prim::bool::True;
}

impl ToBranchBool for typelude_std::std::prim::bool::False {
    type Output = typelude_std::std::prim::bool::False;
}

impl ToBranchBool for typenum::B1 {
    type Output = typelude_std::std::prim::bool::True;
}

impl ToBranchBool for typenum::B0 {
    type Output = typelude_std::std::prim::bool::False;
}

impl<T> ToBranchBool for ELit<T>
where
    T: ToBranchBool,
{
    type Output = <T as ToBranchBool>::Output;
}

pub trait SelectBranch<Then, Else> {
    type Output;
}

impl<Then, Else> SelectBranch<Then, Else> for typelude_std::std::prim::bool::True {
    type Output = Then;
}

impl<Then, Else> SelectBranch<Then, Else> for typelude_std::std::prim::bool::False {
    type Output = Else;
}

pub struct LIfBranch<ThenProg, ElseProg, F>(pub PhantomData<(ThenProg, ElseProg, F)>);

pub struct LIfBranchWithInit<ThenProg, ElseProg, F, InitState>(
    pub PhantomData<(ThenProg, ElseProg, F, InitState)>,
);

impl<ThenProg, ElseProg, F, InitState, Cond, Stack, Locals, Memory, Frames>
    TyFn<VmState<Array<Cond, Stack>, Locals, Memory, Frames>>
    for LIfBranchWithInit<ThenProg, ElseProg, F, InitState>
where
    Cond: ToBranchBool,
    Stack: IsList,
    F: crate::composed::core::MonadState<InitState>,
    ThenProg: crate::composed::vm::program::InterpProgram<F>,
    ElseProg: crate::composed::vm::program::InterpProgram<F>,
    PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>: Eval,
    Then<
        F,
        PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>,
        <ThenProg as crate::composed::vm::program::InterpProgram<F>>::Output,
    >: Eval,
    Then<
        F,
        PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>,
        <ElseProg as crate::composed::vm::program::InterpProgram<F>>::Output,
    >: Eval,
    <Cond as ToBranchBool>::Output: SelectBranch<
            Evaluate<
                Then<
                    F,
                    PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>,
                    <ThenProg as crate::composed::vm::program::InterpProgram<F>>::Output,
                >,
            >,
            Evaluate<
                Then<
                    F,
                    PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>,
                    <ElseProg as crate::composed::vm::program::InterpProgram<F>>::Output,
                >,
            >,
        >,
{
    type Output = <<Cond as ToBranchBool>::Output as SelectBranch<
        Evaluate<
            Then<
                F,
                PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>,
                <ThenProg as crate::composed::vm::program::InterpProgram<F>>::Output,
            >,
        >,
        Evaluate<
            Then<
                F,
                PutVm<F, InitState, VmState<Stack, Locals, Memory, Frames>>,
                <ElseProg as crate::composed::vm::program::InterpProgram<F>>::Output,
            >,
        >,
    >>::Output;
}

impl<ThenProg, ElseProg, F, InitState, Stack, Locals, Memory, Frames>
    TyFn<VmState<Array<typenum::U0, Stack>, Locals, Memory, Frames>>
    for LIfBranchWithInit<ThenProg, ElseProg, F, InitState>
where
    Stack: IsList,
    F: crate::composed::core::MonadError<VmTrap>,
{
    type Output = ThrowVm<F, InvalidCondition>;
}

impl<ThenProg, ElseProg, F, InitState, Stack, Locals, Memory, Frames>
    TyFn<VmState<Array<ELit<typenum::U0>, Stack>, Locals, Memory, Frames>>
    for LIfBranchWithInit<ThenProg, ElseProg, F, InitState>
where
    Stack: IsList,
    F: crate::composed::core::MonadError<VmTrap>,
{
    type Output = ThrowVm<F, InvalidCondition>;
}

pub struct LRunIf<ThenProg, ElseProg, F, InitState>(
    pub PhantomData<(ThenProg, ElseProg, F, InitState)>,
);

impl<ThenProg, ElseProg, F, InitState, State> TyFn<State>
    for LRunIf<ThenProg, ElseProg, F, InitState>
where
    LIfBranchWithInit<ThenProg, ElseProg, F, InitState>: TyFn<State>,
{
    type Output = <LIfBranchWithInit<ThenProg, ElseProg, F, InitState> as TyFn<State>>::Output;
}

impl<ThenProg, ElseProg, InitStack, InitLocals, InitMemory, InitFrames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>>
    for OpIf<ThenProg, ElseProg>
where
    VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>:
        Monad
            + crate::composed::core::MonadWriter<VmTrace>
            + crate::composed::core::MonadState<
                VmState<InitStack, InitLocals, InitMemory, InitFrames>,
            > + crate::composed::core::MonadError<VmTrap>,
    PushTrace<
        VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
        OpIf<ThenProg, ElseProg>,
    >: Sized,
    crate::composed::vm::effects::GetVm<
        VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
        VmState<InitStack, InitLocals, InitMemory, InitFrames>,
    >: Sized,
    Bind<
        VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
        crate::composed::vm::effects::GetVm<
            VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
            VmState<InitStack, InitLocals, InitMemory, InitFrames>,
        >,
        LRunIf<
            ThenProg,
            ElseProg,
            VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
            VmState<InitStack, InitLocals, InitMemory, InitFrames>,
        >,
    >: Eval,
    Then<
        VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
        PushTrace<
            VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
            OpIf<ThenProg, ElseProg>,
        >,
        Bind<
            VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
            crate::composed::vm::effects::GetVm<
                VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
                VmState<InitStack, InitLocals, InitMemory, InitFrames>,
            >,
            LRunIf<
                ThenProg,
                ElseProg,
                VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
                VmState<InitStack, InitLocals, InitMemory, InitFrames>,
            >,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
            PushTrace<
                VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
                OpIf<ThenProg, ElseProg>,
            >,
            Bind<
                VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
                crate::composed::vm::effects::GetVm<
                    VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
                    VmState<InitStack, InitLocals, InitMemory, InitFrames>,
                >,
                LRunIf<
                    ThenProg,
                    ElseProg,
                    VmFx<VmState<InitStack, InitLocals, InitMemory, InitFrames>, Trace, Trap, Req>,
                    VmState<InitStack, InitLocals, InitMemory, InitFrames>,
                >,
            >,
        >,
    >;
}

pub trait LowerWhile<Rest: IsList> {
    type Output: IsList;
}

impl<CondProg, BodyProg, Rest> LowerWhile<Rest> for OpWhile<CondProg, BodyProg>
where
    Rest: IsList,
    BodyProg: Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>,
    <BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output: IsList,
    CondProg: Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Rest,
        >,
    >,
{
    type Output = <CondProg as Concat<
        Array<
            OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>,
            Rest,
        >,
    >>::Output;
}

impl<CondProg, BodyProg, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>
    for OpWhile<CondProg, BodyProg>
where
    Array<OpWhile<CondProg, BodyProg>, Nil>: IsList,
    OpWhile<CondProg, BodyProg>: LowerWhile<Nil>,
    <OpWhile<CondProg, BodyProg> as LowerWhile<Nil>>::Output:
        crate::composed::vm::program::InterpProgram<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            >,
{
    type Output = <<OpWhile<CondProg, BodyProg> as LowerWhile<Nil>>::Output as crate::composed::vm::program::InterpProgram<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
    >>::Output;
}
