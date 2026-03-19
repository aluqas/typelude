use static_assertions::assert_type_eq_all;
use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::Nil,
    tyarray,
};
use typelude_vm::{
    opcode::{OpAdd, OpHostCall, OpPush},
    shared::request::HostRequest,
    vm::{
        effect::world::NoWorld,
        run::direct::ERun,
        state::GetStack,
        step::{StackUnderflow, Step, Suspend, Trap},
        surface::aliases::{MeteredVm, ProgramVm, PureVm, TracedVm},
    },
};
use typenum::{U0, U1, U2, U3};

#[test]
fn pure_machine_run_produces_final_state() {
    type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd];
    type Initial = ProgramVm<Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = PureVm<tyarray![ELit<U3>], Nil, Nil, Nil>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn stack_underflow_is_trapped() {
    type Prog = tyarray![OpAdd];
    type Initial = ProgramVm<Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = Trap<StackUnderflow, Initial>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn traced_machine_accumulates_history() {
    type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd];
    type Initial = TracedVm<Nil, Nil, Nil, Nil, Nil, Prog, Nil>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = TracedVm<
        tyarray![ELit<U3>],
        Nil,
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
    type Prog = tyarray![OpPush<ELit<U1>>];
    type Initial = MeteredVm<Nil, Nil, Nil, Nil, Nil, Prog, Nil, U0, NoWorld>;
    type Final = Evaluate<ERun<ELit<Initial>>>;
    type Expected = Trap<typelude_vm::vm::effect::OutOfFuel, Initial>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn host_call_suspends_machine() {
    struct Print;

    type Initial = PureVm<Nil, Nil, Nil, Nil, Nil, tyarray![OpHostCall<Print>]>;
    type StepResult = <OpHostCall<Print> as Step<Initial>>::Output;
    type Expected =
        Suspend<HostRequest<Print, Nil>, PureVm<Nil, Nil, Nil, Nil, Nil, Nil>>;

    assert_type_eq_all!(StepResult, Expected);
}

#[test]
fn get_stack_trait_is_available_on_final_vm() {
    type Prog = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd];
    type Initial = ProgramVm<Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;

    assert_type_eq_all!(<Final as GetStack>::Output, tyarray![ELit<U3>]);
}
