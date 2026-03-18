use typelude_std::{
    std::{col::array::Nil, debug::trace::Trace},
    tyarray,
    typenum::{U0, U1},
};
use typelude_vm::machine::trace::TracedMachineState;

#[test]
fn test_traced_machine_state_fmt() {
    type Stack = tyarray![U1];
    type Locals = tyarray![U0]; // e.g. one local var with value 0
    type Memory = Nil;
    type History = Nil;
    type CallStack = Nil;
    type Program = Nil;

    type State = TracedMachineState<Stack, Locals, Memory, CallStack, Program, History>;

    let expected = r#"MachineState {
  Stack: [1]
  Locals: [0]
  Memory: []
  History: []
}"#;

    assert_eq!(<State as Trace>::fmt(), expected);
}
