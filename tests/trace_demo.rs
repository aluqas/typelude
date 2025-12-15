#![recursion_limit = "1024"]
use typelude::{
    eval::Evaluator,
    machine::{execution::ERun, state::MachineState},
    program,
    std::array::TyNil,
    std::trace::Trace,
};

#[test]
fn test_trace_output() {
    // A simple program to demonstrate tracing
    // Push 3, Push 5, Add
    type Prog = program! {
        (push 3)
        (push 5)
        (add)
    };

    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluator<ERun<InitialState>>;

    let trace_output = FinalState::fmt();

    // Print it so we can see it in logs (use --nocapture to view)
    println!("Trace Output:\n{}", trace_output);

    // Verify it contains expected information
    // History should contain instructions in reverse order of execution (latest first)
    // [Add, Push(5), Push(3)]

    // Check Stack: [8]
    assert!(trace_output.contains("Stack: [8]"));

    // Check History content
    // Note: My Trace implementation for list uses `[A, B, C]`.
    // History is constructed by appending: `TyArray<Inst, History>`.
    // Initial history is `[]`.
    // 1. Push(3) -> `[Push(3)]`
    // 2. Push(5) -> `[Push(5), Push(3)]`
    // 3. Add -> `[Add, Push(5), Push(3)]`

    assert!(trace_output.contains("Add, Push(5), Push(3)"));
}
