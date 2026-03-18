use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, TyFn},
    std::col::array::{Array, Concat, IsList, Nil},
};

use crate::{
    composed::{
        core::{Bind, Monad},
        interpret::InterpInstr,
        vm::{
            effects::{InvalidCondition, PushTrace, PutVm, Then, ThrowVm, VmFx, VmTrap, VmTrace},
            state::VmState,
        },
    },
    machine::instr::core::{OpIf, OpWhile},
};

pub struct LIfBranch<ThenProg, ElseProg, F>(pub PhantomData<(ThenProg, ElseProg, F)>);

impl<ThenProg, ElseProg, F, Stack, Locals, Memory, Frames>
    TyFn<VmState<Array<typenum::B1, Stack>, Locals, Memory, Frames>> for LIfBranch<ThenProg, ElseProg, F>
where
    Stack: IsList,
    F: crate::composed::core::MonadState<VmState<Array<typenum::B1, Stack>, Locals, Memory, Frames>>,
    ThenProg: crate::composed::vm::program::InterpProgram<F>,
    PutVm<
        F,
        VmState<Array<typenum::B1, Stack>, Locals, Memory, Frames>,
        VmState<Stack, Locals, Memory, Frames>,
    >: Eval,
    Then<
        F,
        PutVm<
            F,
            VmState<Array<typenum::B1, Stack>, Locals, Memory, Frames>,
            VmState<Stack, Locals, Memory, Frames>,
        >,
        <ThenProg as crate::composed::vm::program::InterpProgram<F>>::Output,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            F,
            PutVm<
                F,
                VmState<Array<typenum::B1, Stack>, Locals, Memory, Frames>,
                VmState<Stack, Locals, Memory, Frames>,
            >,
            <ThenProg as crate::composed::vm::program::InterpProgram<F>>::Output,
        >,
    >;
}

impl<ThenProg, ElseProg, F, Stack, Locals, Memory, Frames>
    TyFn<VmState<Array<typenum::B0, Stack>, Locals, Memory, Frames>> for LIfBranch<ThenProg, ElseProg, F>
where
    Stack: IsList,
    F: crate::composed::core::MonadState<VmState<Array<typenum::B0, Stack>, Locals, Memory, Frames>>,
    ElseProg: crate::composed::vm::program::InterpProgram<F>,
    PutVm<
        F,
        VmState<Array<typenum::B0, Stack>, Locals, Memory, Frames>,
        VmState<Stack, Locals, Memory, Frames>,
    >: Eval,
    Then<
        F,
        PutVm<
            F,
            VmState<Array<typenum::B0, Stack>, Locals, Memory, Frames>,
            VmState<Stack, Locals, Memory, Frames>,
        >,
        <ElseProg as crate::composed::vm::program::InterpProgram<F>>::Output,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            F,
            PutVm<
                F,
                VmState<Array<typenum::B0, Stack>, Locals, Memory, Frames>,
                VmState<Stack, Locals, Memory, Frames>,
            >,
            <ElseProg as crate::composed::vm::program::InterpProgram<F>>::Output,
        >,
    >;
}

impl<ThenProg, ElseProg, F, Stack, Locals, Memory, Frames>
    TyFn<VmState<Array<typenum::U0, Stack>, Locals, Memory, Frames>> for LIfBranch<ThenProg, ElseProg, F>
where
    Stack: IsList,
    F: crate::composed::core::MonadError<VmTrap>,
{
    type Output = ThrowVm<F, InvalidCondition>;
}

pub struct LRunIf<ThenProg, ElseProg, F>(pub PhantomData<(ThenProg, ElseProg, F)>);

impl<ThenProg, ElseProg, F, State> TyFn<State> for LRunIf<ThenProg, ElseProg, F>
where
    LIfBranch<ThenProg, ElseProg, F>: TyFn<State>,
{
    type Output = <LIfBranch<ThenProg, ElseProg, F> as TyFn<State>>::Output;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpIf<ThenProg, ElseProg>
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>
        + crate::composed::core::MonadError<VmTrap>,
    PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpIf<ThenProg, ElseProg>>: Sized,
    crate::composed::vm::effects::GetVm<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        VmState<Stack, Locals, Memory, Frames>,
    >: Sized,
    Bind<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        crate::composed::vm::effects::GetVm<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
        >,
        LRunIf<ThenProg, ElseProg, VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>,
    >: Eval,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpIf<ThenProg, ElseProg>>,
        Bind<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            crate::composed::vm::effects::GetVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
            >,
            LRunIf<ThenProg, ElseProg, VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpIf<ThenProg, ElseProg>>,
            Bind<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                crate::composed::vm::effects::GetVm<
                    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                    VmState<Stack, Locals, Memory, Frames>,
                >,
                LRunIf<ThenProg, ElseProg, VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>,
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
        Array<OpIf<<BodyProg as Concat<Array<OpWhile<CondProg, BodyProg>, Nil>>>::Output, Nil>, Rest>,
    >>::Output;
}

impl<CondProg, BodyProg, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpWhile<CondProg, BodyProg>
where
    Array<OpWhile<CondProg, BodyProg>, Nil>: IsList,
    OpWhile<CondProg, BodyProg>: LowerWhile<Nil>,
    <OpWhile<CondProg, BodyProg> as LowerWhile<Nil>>::Output:
        crate::composed::vm::program::InterpProgram<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>,
{
    type Output = <<OpWhile<CondProg, BodyProg> as LowerWhile<Nil>>::Output as crate::composed::vm::program::InterpProgram<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
    >>::Output;
}
