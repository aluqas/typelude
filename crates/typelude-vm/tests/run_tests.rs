use static_assertions::assert_type_eq_all;
use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::Nil,
    tyarray,
};
use typelude_vm::machine::{
    core::CoreState,
    effects::{
        Effects, PureEffects,
        fuel::{MeteredFuel, OutOfFuel},
        io::{HostRequest, SuspendIoPolicy},
        trace::NoTrace,
        trap::TrapAsResult,
    },
    instr::core::{OpAdd, OpHostCall, OpPush},
    machine::Machine,
    meta::{DefaultMeta, NoWorld, VmMeta},
    result::{Suspend, Trap},
    run::ERun,
    state::MachineState,
    step::{StackUnderflow, Step},
    trace::TracedMachineState,
};
use typenum::{U0, U1, U2, U3};

#[test]
fn pure_machine_run_produces_final_state() {
    type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd];
    type Initial = MachineState<Nil, Nil, Nil, Nil, Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = MachineState<tyarray![ELit<U3>], Nil, Nil, Nil, Nil>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn stack_underflow_is_trapped() {
    type Prog = tyarray![OpAdd];
    type Initial = MachineState<Nil, Nil, Nil, Nil, Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = Trap<StackUnderflow, Initial>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn traced_machine_accumulates_history() {
    type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd];
    type Initial = TracedMachineState<Nil, Nil, Nil, Nil, Prog, Nil>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = TracedMachineState<
        tyarray![ELit<U3>],
        Nil,
        Nil,
        Nil,
        Nil,
        tyarray![OpAdd, OpPush<ELit<U2>>, OpPush<ELit<U1>>],
    >;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn metered_effects_trap_when_fuel_is_empty() {
    type Fx = Effects<NoTrace, MeteredFuel, TrapAsResult, SuspendIoPolicy>;
    type Prog = tyarray![OpPush<ELit<U1>>];
    type Initial = Machine<CoreState<Nil, Nil, Nil, Nil, Nil, Prog>, VmMeta<Nil, U0, NoWorld>, Fx>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = Trap<OutOfFuel, Initial>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn host_call_suspends_machine() {
    struct Print;

    type Fx = PureEffects;
    type Initial =
        Machine<CoreState<Nil, Nil, Nil, Nil, Nil, tyarray![OpHostCall<Print>]>, DefaultMeta, Fx>;
    type StepResult = <OpHostCall<Print> as Step<Initial>>::Output;
    type Expected = Suspend<
        HostRequest<Print, Nil>,
        Machine<CoreState<Nil, Nil, Nil, Nil, Nil, Nil>, DefaultMeta, Fx>,
    >;

    assert_type_eq_all!(StepResult, Expected);
}
