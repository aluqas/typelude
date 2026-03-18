use core::marker::PhantomData;

use typelude_std::std::col::array::Nil;

use crate::composed::core::{
    Bind, EitherT, IdK, MonadError, MonadState, MonadSuspend, MonadWriter, Pure, StateT, SuspendT,
    Unit, WriterT,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmTrace;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmTrap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmRequest;

#[derive(Debug)]
pub struct VmTraceEvent<Instr>(pub PhantomData<Instr>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackUnderflow;

#[derive(Debug)]
pub struct BadLocalIndex<Idx>(pub PhantomData<Idx>);

#[derive(Debug)]
pub struct BadMemoryIndex<Idx>(pub PhantomData<Idx>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReturnUnderflow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidCondition;

pub type VmFx<State, Trace = VmTrace, Trap = VmTrap, Req = VmRequest> =
    SuspendT<Req, EitherT<Trap, StateT<State, WriterT<Trace, IdK>>>>;

pub type PushTrace<F, Instr> = <F as MonadWriter<VmTrace>>::Tell<VmTraceEvent<Instr>>;
pub type ThrowVm<F, Reason> = <F as MonadError<VmTrap>>::Throw<Reason>;
pub type YieldVm<F, Request> = <F as MonadSuspend<VmRequest>>::Suspend<Request>;
pub type GetVm<F, State> = <F as MonadState<State>>::Get;
pub type PutVm<F, State, NewState> = <F as MonadState<State>>::Put<NewState>;
pub type ModifyVm<F, State, Func> = <F as MonadState<State>>::Modify<Func>;
pub type ReturnVm<F, A> = Pure<F, A>;
pub type Then<F, MA, MB> = Bind<F, MA, crate::composed::core::traits::LConst<MB>>;

pub type EmptyTrace = Nil;
pub type VmUnit<F> = Pure<F, Unit>;
