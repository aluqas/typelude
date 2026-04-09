use typelude_std::{
    std::{col::array::Nil, debug::trace::Trace},
    tyarray,
    typenum::{U0, U1},
};
use typelude_vm::vm::semantics::state::VmState;

#[test]
fn traced_machine_state_format_is_stable() {
    type Stack = tyarray![U1];
    type Locals = tyarray![U0];
    type Memory = Nil;
    type CallStack = Nil;
    type Program = Nil;

    type State = VmState<Stack, Locals, Memory, CallStack, Program>;

    let expected = r#"VmState {
  Stack: [1]
  Locals: [0]
  Memory: []
}"#;

    assert_eq!(<State as Trace>::fmt(), expected);
}
