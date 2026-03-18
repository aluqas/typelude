#![recursion_limit = "4096"]

use static_assertions::assert_type_eq_all;
use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::{ELen, Nil},
    tyarray,
};
use typelude_vm::{
    composed::{
        core::{
            Monad, MonadWriter,
            either_t::ERunEither,
            id::{ERunId, IdK},
            state_t::{ERunState, StateT},
            suspend_t::ERunSuspend,
            traits::{Done as SDone, Ok, Pair, Pure, Unit},
            writer_t::{ERunWriter, WriterT},
        },
        vm::{
            effects::{InvalidCondition, StackUnderflow, VmFx, VmLog, VmTraceEvent},
            outcome::{Done, Raised, Suspended},
            program::EInterpProgram,
            run::ERunVm,
            state::VmState,
        },
    },
    machine::instr::core::{
        OpAdd, OpGetLocal, OpHostCall, OpIf, OpLet, OpLt, OpPush, OpSetLocal, OpWhile,
    },
};
use typenum::{U0, U1, U2, U3, U10, U55, U89, U151, U152};


type FibState = VmState<Nil, Nil, Nil, Nil>;
type FibFinalState =
    VmState<tyarray![ELit<U55>], tyarray![ELit<U10>, ELit<U89>, ELit<U55>], Nil, Nil>;
type FibCond<N> = tyarray![OpPush<ELit<N>>, OpGetLocal<ELit<U0>>, OpLt];
type FibBody = tyarray![
    OpGetLocal<ELit<U2>>,
    OpGetLocal<ELit<U1>>,
    OpAdd,
    OpGetLocal<ELit<U1>>,
    OpSetLocal<ELit<U2>>,
    OpSetLocal<ELit<U1>>,
    OpPush<ELit<U1>>,
    OpGetLocal<ELit<U0>>,
    OpAdd,
    OpSetLocal<ELit<U0>>
];
type FibProgram10 = tyarray![
    OpPush<ELit<U0>>,
    OpPush<ELit<U1>>,
    OpPush<ELit<U0>>,
    OpLet,
    OpLet,
    OpLet,
    OpWhile<FibCond<U10>, FibBody>,
    OpGetLocal<ELit<U2>>
];
type FibProgram10WithHost<Sig> = tyarray![
    OpPush<ELit<U0>>,
    OpPush<ELit<U1>>,
    OpPush<ELit<U0>>,
    OpLet,
    OpLet,
    OpLet,
    OpWhile<FibCond<U10>, FibBody>,
    OpGetLocal<ELit<U2>>,
    OpHostCall<Sig>
];

trait OutcomeState {
    type Output;
}

impl<A, State, Trace> OutcomeState for Done<A, State, Trace> {
    type Output = State;
}

impl<Reason, State, Trace> OutcomeState for Raised<Reason, State, Trace> {
    type Output = State;
}

impl<Request, State, Trace> OutcomeState for Suspended<Request, State, Trace> {
    type Output = State;
}

trait OutcomeTrace {
    type Output;
}

impl<A, State, Trace> OutcomeTrace for Done<A, State, Trace> {
    type Output = Trace;
}

impl<Reason, State, Trace> OutcomeTrace for Raised<Reason, State, Trace> {
    type Output = Trace;
}

impl<Request, State, Trace> OutcomeTrace for Suspended<Request, State, Trace> {
    type Output = Trace;
}

trait OutcomeRequest {
    type Output;
}

impl<Request, State, Trace> OutcomeRequest for Suspended<Request, State, Trace> {
    type Output = Request;
}

#[derive(Debug)]
struct FibStart<N>(core::marker::PhantomData<N>);

#[derive(Debug)]
struct FibDone<N, Value>(core::marker::PhantomData<(N, Value)>);

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
    type Expected = Done<
        Unit,
        VmState<tyarray![ELit<U3>], Nil, Nil, Nil>,
        tyarray![
            VmTraceEvent<OpPush<ELit<U1>>>,
            VmTraceEvent<OpPush<ELit<U2>>>,
            VmTraceEvent<OpAdd>
        ],
    >;
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
    type Expected = Suspended<
        typelude_vm::composed::interpret::HostRequest<Print>,
        State,
        tyarray![VmTraceEvent<OpHostCall<Print>>],
    >;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_if_picks_branch() {
    type State = VmState<tyarray![typenum::B1], Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<
        EInterpProgram<tyarray![OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>], F>,
    >;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Done<
        Unit,
        VmState<tyarray![ELit<U1>], Nil, Nil, Nil>,
        tyarray![
            VmTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>,
            VmTraceEvent<OpPush<ELit<U1>>>
        ],
    >;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_if_invalid_condition_traps() {
    type State = VmState<tyarray![U0], Nil, Nil, Nil>;
    type F = VmFx<State>;
    type Prog = Evaluate<
        EInterpProgram<tyarray![OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>], F>,
    >;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Raised<
        InvalidCondition,
        State,
        tyarray![VmTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>],
    >;
    assert_type_eq_all!(Out, Expected);
}

#[test]
fn composed_vm_runs_fibonacci_10() {
    type F = VmFx<FibState>;
    type Prog = Evaluate<EInterpProgram<FibProgram10, F>>;
    type Out = Evaluate<ERunVm<FibState, Prog>>;
    type FinalState = <Out as OutcomeState>::Output;

    assert_type_eq_all!(FinalState, FibFinalState);
}

#[test]
fn composed_vm_fibonacci_10_trace_has_expected_length() {
    type F = VmFx<FibState>;
    type Prog = Evaluate<EInterpProgram<FibProgram10, F>>;
    type Out = Evaluate<ERunVm<FibState, Prog>>;
    type Trace = <Out as OutcomeTrace>::Output;
    type TraceLen = Evaluate<ELen<ELit<Trace>>>;

    assert_type_eq_all!(TraceLen, U151);
}

#[test]
fn composed_vm_fibonacci_10_then_host_call_suspends_with_final_state() {
    struct Print;

    type F = VmFx<FibState>;
    type Prog = Evaluate<EInterpProgram<FibProgram10WithHost<Print>, F>>;
    type Out = Evaluate<ERunVm<FibState, Prog>>;
    type FinalState = <Out as OutcomeState>::Output;
    type Request = <Out as OutcomeRequest>::Output;
    type Trace = <Out as OutcomeTrace>::Output;
    type TraceLen = Evaluate<ELen<ELit<Trace>>>;

    assert_type_eq_all!(FinalState, FibFinalState);
    assert_type_eq_all!(Request, typelude_vm::composed::interpret::HostRequest<Print>);
    assert_type_eq_all!(TraceLen, U152);
}

#[test]
fn composed_vm_fibonacci_10_can_accumulate_log_and_trace() {
    type F = WriterT<VmLog, IdK>;
    type Logged = <F as Monad>::Bind<
        <F as MonadWriter<VmLog>>::Tell<FibStart<U10>>,
        typelude_vm::composed::core::traits::LConst<
            <F as Monad>::Bind<
                <F as MonadWriter<VmLog>>::Tell<FibDone<U10, U55>>,
                typelude_vm::composed::core::traits::LConst<Pure<F, Unit>>,
            >,
        >,
    >;
    type Raw = Evaluate<ERunId<ERunWriter<Logged>>>;
    type Result = Raw;
    type ExpectedLog = tyarray![FibStart<U10>, FibDone<U10, U55>];

    assert_type_eq_all!(Result, Pair<Unit, ExpectedLog>);
}
