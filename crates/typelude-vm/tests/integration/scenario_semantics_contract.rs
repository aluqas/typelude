use static_assertions::assert_type_eq_all;
use typelude_std::{
    core::{ELit, Evaluate},
    std::{col::array::Nil, prim::bool::False},
    tyarray,
};
use typelude_vm::{
    opcode::{
        control::{OpIf, OpWhile},
        host::OpHostCall,
        memory::OpStore,
        numeric::{OpGt, OpLt, OpSub},
        stack::OpPush,
    },
    vm::{
        protocol::request::HostRequest,
        runtime::effects::trace::VmTraceEvent,
        semantics::{
            state::VmState,
            step::{StepInstr, StepSuspend},
        },
    },
};
use typenum::{U1, U2, U3, U5, U7};

use crate::support::{OutcomeState, OutcomeTrace, ProgramRun, Run, StateMemory, StateStack};

struct Print;

#[test]
fn sub_uses_top_as_lhs() {
    type Program = tyarray![OpPush<ELit<U2>>, OpPush<ELit<U5>>, OpSub];
    type Out = ProgramRun<Program>;
    type State = <Out as OutcomeState>::Output;

    assert_type_eq_all!(<State as StateStack>::Output, tyarray![ELit<U3>]);
}

#[test]
fn lt_and_gt_use_current_operand_order() {
    type LtProgram = tyarray![OpPush<ELit<U2>>, OpPush<ELit<U5>>, OpLt];
    type GtProgram = tyarray![OpPush<ELit<U2>>, OpPush<ELit<U5>>, OpGt];
    type LtState = <<ProgramRun<LtProgram> as OutcomeState>::Output as StateStack>::Output;
    type GtState = <<ProgramRun<GtProgram> as OutcomeState>::Output as StateStack>::Output;

    assert_type_eq_all!(LtState, tyarray![ELit<typelude_std::std::prim::bool::False>]);
    assert_type_eq_all!(GtState, tyarray![ELit<typelude_std::std::prim::bool::True>]);
}

#[test]
fn store_consumes_value_then_address() {
    type Initial = VmState<
        tyarray![ELit<U7>, ELit<U1>],
        Nil,
        tyarray![ELit<U2>, ELit<U3>],
        Nil,
        tyarray![OpStore],
    >;
    type Out = Run<Initial>;
    type State = <Out as OutcomeState>::Output;

    assert_type_eq_all!(<State as StateStack>::Output, Nil);
    assert_type_eq_all!(<State as StateMemory>::Output, tyarray![ELit<U2>, ELit<U7>]);
}

#[test]
fn host_call_pure_step_suspends_with_advanced_state() {
    type Initial =
        VmState<tyarray![ELit<U5>], Nil, Nil, Nil, tyarray![OpHostCall<Print>, OpPush<ELit<U1>>]>;
    type Actual = Evaluate<<OpHostCall<Print> as StepInstr<Initial>>::Output>;
    type Expected = StepSuspend<
        HostRequest<Print, tyarray![ELit<U5>]>,
        VmState<tyarray![ELit<U5>], Nil, Nil, Nil, tyarray![OpPush<ELit<U1>>]>,
    >;

    assert_type_eq_all!(Actual, Expected);
}

#[test]
fn while_lowering_matches_runtime_trace_and_result() {
    type Cond = tyarray![OpPush<ELit<False>>];
    type Body = tyarray![OpPush<ELit<U1>>];
    type WhileProgram = tyarray![OpWhile<Cond, Body>, OpPush<ELit<U2>>];
    type LoweredProgram = tyarray![
        OpPush<ELit<False>>,
        OpIf<tyarray![OpPush<ELit<U1>>, OpWhile<Cond, Body>], Nil>,
        OpPush<ELit<U2>>
    ];
    type WhileOut = ProgramRun<WhileProgram>;
    type LoweredOut = ProgramRun<LoweredProgram>;

    assert_type_eq_all!(
        <<WhileOut as OutcomeState>::Output as StateStack>::Output,
        <<LoweredOut as OutcomeState>::Output as StateStack>::Output
    );
    assert_type_eq_all!(<WhileOut as OutcomeTrace>::Output, tyarray![
        VmTraceEvent<OpWhile<Cond, Body>>,
        VmTraceEvent<OpPush<ELit<False>>>,
        VmTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>, OpWhile<Cond, Body>], Nil>>,
        VmTraceEvent<OpPush<ELit<U2>>>
    ]);
}
