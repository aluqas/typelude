use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, tyarray};
use typelude_vm::{
    opcode::{host::OpHostCall, numeric::OpAdd, stack::OpPush},
    vm::protocol::request::HostSignature,
};
use typenum::{U1, U2, U3};

use crate::support::{CoreTraceLen, OutcomeState, ProgramRun, Resume, SourceTraceLen, StateStack};

struct ResumeTwo;

impl HostSignature for ResumeTwo {
    type Response = U2;
}

type ResumeProgram = tyarray![OpPush<ELit<U1>>, OpHostCall<ResumeTwo>, OpAdd];

#[test]
fn host_resume_pushes_response_and_continues() {
    type SuspendedOut = ProgramRun<ResumeProgram>;
    type ResumedOut = Resume<SuspendedOut, U2>;
    type ResumedState = <ResumedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ResumedState as StateStack>::Output, tyarray![ELit<U3>]);
    assert_type_eq_all!(SourceTraceLen<ResumedOut>, U3);
    assert_type_eq_all!(CoreTraceLen<ResumedOut>, U3);
}
