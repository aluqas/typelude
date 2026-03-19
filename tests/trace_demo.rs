#![recursion_limit = "1024"]

use typelude::{
    Evaluate,
    core::ELit,
    std::{col::array::Nil, debug::trace::Trace},
    tyarray,
    typenum::{U3, U5},
    vm::{
        opcode::{OpAdd, OpPush},
        vm::{run::direct::ERun, surface::aliases::TracedVm},
    },
};

#[test]
fn test_trace_output() {
    type Prog = tyarray![OpPush<ELit<U3>>, OpPush<ELit<U5>>, OpAdd];
    type InitialState = TracedVm<Nil, Nil, Nil, Nil, Nil, Prog, Nil>;
    type FinalState = Evaluate<ERun<ELit<InitialState>>>;

    let trace_output = FinalState::fmt();

    assert!(trace_output.contains("Stack: [8]"));
    assert!(trace_output.contains("Add, Push(5), Push(3)"));
}
