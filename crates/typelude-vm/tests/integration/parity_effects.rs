use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::prim::bool::True, tyarray};
use typelude_vm::{
    opcode::{OpAdd, OpHostCall, OpIf, OpPush},
    vm::surface::aliases::TracedVm,
};
use typenum::{U1, U2, U3, U4};

use crate::support::{
    ComposedRun, ComposedTraceLen, DirectHistoryLen, DirectRun, DirectTracedRun,
    EmptyComposedState, OutcomeRequest, OutcomeState, RequestSig, StateStack,
};

struct Print;

type HostProgram = tyarray![OpPush<ELit<U1>>, OpPush<ELit<U2>>, OpAdd, OpHostCall<Print>];
type IfTraceProgram =
    tyarray![OpPush<ELit<True>>, OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>];

#[test]
fn host_suspend_matches_state_and_request_sig() {
    type DirectInitial = TracedVm<
        typelude_std::std::col::array::Nil,
        typelude_std::std::col::array::Nil,
        typelude_std::std::col::array::Nil,
        typelude_std::std::col::array::Nil,
        typelude_std::std::col::array::Nil,
        HostProgram,
        typelude_std::std::col::array::Nil,
    >;
    type DirectOut = DirectRun<DirectInitial>;
    type DirectState = <DirectOut as OutcomeState>::Output;
    type DirectSig = <<DirectOut as OutcomeRequest>::Output as RequestSig>::Output;

    type ComposedOut = ComposedRun<EmptyComposedState, HostProgram>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;
    type ComposedSig = <<ComposedOut as OutcomeRequest>::Output as RequestSig>::Output;

    assert_type_eq_all!(
        <DirectState as StateStack>::Output,
        <ComposedState as StateStack>::Output,
        tyarray![ELit<U3>]
    );
    assert_type_eq_all!(DirectSig, ComposedSig, Print);
    assert_type_eq_all!(DirectHistoryLen<DirectState>, ComposedTraceLen<ComposedOut>, U4);
}

#[test]
fn traced_if_records_only_taken_branch_on_both_runners() {
    type DirectOut = DirectTracedRun<IfTraceProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    type ComposedOut = ComposedRun<EmptyComposedState, IfTraceProgram>;

    assert_type_eq_all!(DirectHistoryLen<DirectState>, ComposedTraceLen<ComposedOut>, U3);
}
