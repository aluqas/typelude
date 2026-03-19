use typelude_std::{
    std::{col::array::Nil, debug::trace::Trace},
    tyarray,
    typenum::{U0, U1},
};
use typelude_vm::vm::surface::aliases::TracedVm;

#[test]
fn traced_machine_state_format_is_stable() {
    type Stack = tyarray![U1];
    type Locals = tyarray![U0];
    type Memory = Nil;
    type History = Nil;
    type CallStack = Nil;
    type Program = Nil;
    type Labels = Nil;

    type State = TracedVm<Stack, Locals, Memory, CallStack, Labels, Program, History>;

    let expected = r#"MachineState {
  Stack: [1]
  Locals: [0]
  Memory: []
  History: []
}"#;

    assert_eq!(<State as Trace>::fmt(), expected);
}
