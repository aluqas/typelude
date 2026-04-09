use static_assertions::assert_type_eq_all;
use typelude_bool::{False, True};
use typelude_col::{TTerm, tarr};
use typelude_std::core::{Apply, Evaluate};
use typelude_vm::{
    opcode::{
        control::{OpIf, OpWhile},
        host::OpHostCall,
        memory::OpStore,
        numeric::{OpGt, OpLt, OpSub},
        stack::OpPush,
    },
    vm::{
        protocol::request::{HostRequest, HostSignature},
        runtime::effects::trace::{CoreTraceEvent, SourceTraceEvent},
        semantics::{state::VmState, step::StepSuspend},
        value::Lit,
    },
};
use typenum::{U1, U2, U3, U5, U7};

use crate::support::{
    OutcomeCoreTrace, OutcomeSourceTrace, OutcomeState, ProgramRun, Run, StateMemory, StateStack,
};

struct Print;

impl HostSignature for Print {
    type Response = U1;
}

#[test]
fn sub_uses_top_as_lhs() {
    type Program = tarr![OpPush<Lit<U2>>, OpPush<Lit<U5>>, OpSub];
    type Out = ProgramRun<Program>;
    type State = <Out as OutcomeState>::Output;

    assert_type_eq_all!(<State as StateStack>::Output, tarr![Lit<U3>]);
}

#[test]
fn lt_and_gt_use_current_operand_order() {
    type LtProgram = tarr![OpPush<Lit<U2>>, OpPush<Lit<U5>>, OpLt];
    type GtProgram = tarr![OpPush<Lit<U2>>, OpPush<Lit<U5>>, OpGt];
    type LtState = <<ProgramRun<LtProgram> as OutcomeState>::Output as StateStack>::Output;
    type GtState = <<ProgramRun<GtProgram> as OutcomeState>::Output as StateStack>::Output;

    assert_type_eq_all!(LtState, tarr![Lit<False>]);
    assert_type_eq_all!(GtState, tarr![Lit<True>]);
}

#[test]
fn store_consumes_value_then_address() {
    type Initial =
        VmState<tarr![Lit<U7>, Lit<U1>], TTerm, tarr![Lit<U2>, Lit<U3>], TTerm, tarr![OpStore]>;
    type Out = Run<Initial>;
    type State = <Out as OutcomeState>::Output;

    assert_type_eq_all!(<State as StateStack>::Output, TTerm);
    assert_type_eq_all!(<State as StateMemory>::Output, tarr![Lit<U2>, Lit<U7>]);
}

#[test]
fn host_call_pure_step_suspends_with_advanced_state() {
    type Initial =
        VmState<tarr![Lit<U5>], TTerm, TTerm, TTerm, tarr![OpHostCall<Print>, OpPush<Lit<U1>>]>;
    type Actual = Evaluate<Apply<OpHostCall<Print>, Initial>>;
    type Expected = StepSuspend<
        HostRequest<Print, tarr![Lit<U5>], U1>,
        VmState<tarr![Lit<U5>], TTerm, TTerm, TTerm, tarr![OpPush<Lit<U1>>]>,
    >;

    assert_type_eq_all!(Actual, Expected);
}

#[test]
fn while_lowering_matches_runtime_trace_and_result() {
    type Cond = tarr![OpPush<Lit<False>>];
    type Body = tarr![OpPush<Lit<U1>>];
    type WhileProgram = tarr![OpWhile<Cond, Body>, OpPush<Lit<U2>>];
    type LoweredProgram = tarr![
        OpPush<Lit<False>>,
        OpIf<tarr![OpPush<Lit<U1>>, OpWhile<Cond, Body>], TTerm>,
        OpPush<Lit<U2>>
    ];
    type WhileOut = ProgramRun<WhileProgram>;
    type LoweredOut = ProgramRun<LoweredProgram>;

    assert_type_eq_all!(
        <<WhileOut as OutcomeState>::Output as StateStack>::Output,
        <<LoweredOut as OutcomeState>::Output as StateStack>::Output
    );
    assert_type_eq_all!(<WhileOut as OutcomeSourceTrace>::Output, tarr![
        SourceTraceEvent<OpWhile<Cond, Body>>,
        SourceTraceEvent<OpPush<Lit<False>>>,
        SourceTraceEvent<OpIf<tarr![OpPush<Lit<U1>>, OpWhile<Cond, Body>], TTerm>>,
        SourceTraceEvent<OpPush<Lit<U2>>>
    ]);
    assert_type_eq_all!(<WhileOut as OutcomeCoreTrace>::Output, tarr![
        CoreTraceEvent<OpPush<Lit<False>>>,
        CoreTraceEvent<OpIf<tarr![OpPush<Lit<U1>>, OpWhile<Cond, Body>], TTerm>>,
        CoreTraceEvent<OpPush<Lit<U2>>>
    ]);
}
