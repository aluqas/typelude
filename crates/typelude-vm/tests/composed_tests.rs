use static_assertions::assert_type_eq_all;
use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::Nil,
    tyarray,
};
use typenum::{U0, U1, U2, U3};

use typelude_vm::composed::{
    core::{
        either_t::ERunEither,
        id::{ERunId, IdK},
        state_t::{ERunState, StateT},
        suspend_t::ERunSuspend,
        traits::{Done as SDone, Ok, Pair, Pure, Unit},
        writer_t::{ERunWriter, WriterT},
        Monad, MonadWriter,
    },
    vm::{
        effects::{InvalidCondition, StackUnderflow, VmFx, VmTraceEvent},
        outcome::{Done, Raised, Suspended},
        program::EInterpProgram,
        run::ERunVm,
        state::VmState,
    },
};
use typelude_vm::machine::instr::core::{OpAdd, OpHostCall, OpIf, OpPush};

#[test]
fn state_writer_either_suspend_stack_runs() {
    type Base = WriterT<(), IdK>;
    type WithState = StateT<U0, Base>;
    type WithEither = typelude_vm::composed::core::EitherT<(), WithState>;
    type WithSuspend = typelude_vm::composed::core::SuspendT<(), WithEither>;
    type Prog = Pure<WithSuspend, U1>;

    type Out = Evaluate<ERunId<ERunWriter<ERunState<U0, ERunEither<ERunSuspend<Prog>>>>>>;
    type Expected = Pair<Pair<Ok<SDone<U1>>, U0>, Nil>;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn writer_tell_accumulates_log() {
    type F = WriterT<(), IdK>;
    type P = <F as Monad>::Bind<
        <F as MonadWriter<()>>::Tell<U1>,
        typelude_vm::composed::core::traits::LConst<<F as MonadWriter<()>>::Tell<U2>>,
    >;
    type Out = Evaluate<ERunId<ERunWriter<P>>>;
    type Expected = Pair<Unit, tyarray![U1, U2]>;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_runs_push_push_add() {
    type State = VmState<Nil, Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<EInterpProgram<tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd], F>>;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Done<Unit, VmState<tyarray![U3], Nil, Nil, Nil>, tyarray![VmTraceEvent<OpPush<ELit<U1>>>, VmTraceEvent<OpPush<ELit<U2>>>, VmTraceEvent<OpAdd>]>;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_traps_on_underflow() {
    type State = VmState<Nil, Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<EInterpProgram<tyarray![OpAdd], F>>;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Raised<StackUnderflow, State, tyarray![VmTraceEvent<OpAdd>]>;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_suspends_on_host_call() {
    struct Print;
    type State = VmState<Nil, Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<EInterpProgram<tyarray![OpHostCall<Print>], F>>;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Suspended<typelude_vm::composed::interpret::HostRequest<Print>, State, tyarray![VmTraceEvent<OpHostCall<Print>>]>;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_if_picks_branch() {
    type State = VmState<tyarray![typenum::B1], Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<EInterpProgram<tyarray![OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>], F>>;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Done<
        Unit,
        VmState<tyarray![ELit<U1>], Nil, Nil, Nil>,
        tyarray![VmTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>, VmTraceEvent<OpPush<ELit<U1>>>],
    >;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_if_invalid_condition_traps() {
    type State = VmState<tyarray![U0], Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<EInterpProgram<tyarray![OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>], F>>;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Raised<
        InvalidCondition,
        State,
        tyarray![VmTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>],
    >;
    assert_type_eq_all!(Out, Expected);
}
