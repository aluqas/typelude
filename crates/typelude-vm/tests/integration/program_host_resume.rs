use static_assertions::assert_type_eq_all;
use typelude_col::tarr;
use typelude_vm::{
    opcode::{host::OpHostCall, numeric::OpAdd, stack::OpPush},
    vm::{protocol::request::HostSignature, value::Lit},
};
use typenum::{U1, U2, U3};

use crate::support::{
    CoreTraceLen, N3, OutcomeState, ProgramRun, Resume, SourceTraceLen, StateStack,
};

struct ResumeTwo;

impl HostSignature for ResumeTwo {
    type Response = U2;
}

type ResumeProgram = tarr![OpPush<Lit<U1>>, OpHostCall<ResumeTwo>, OpAdd];

#[test]
fn host_resume_pushes_response_and_continues() {
    type SuspendedOut = ProgramRun<ResumeProgram>;
    type ResumedOut = Resume<SuspendedOut, U2>;
    type ResumedState = <ResumedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ResumedState as StateStack>::Output, tarr![Lit<U3>]);
    assert_type_eq_all!(SourceTraceLen<ResumedOut>, N3);
    assert_type_eq_all!(CoreTraceLen<ResumedOut>, N3);
}
