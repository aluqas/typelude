use core::marker::PhantomData;

use typelude_col::{TTerm, tarr};
use typelude_vm::vm::semantics::state::VmState;
use typenum::{U0, U1};

#[test]
fn machine_state_debug_format_is_available() {
    type Stack = tarr![U1];
    type Locals = tarr![U0];
    type Memory = TTerm;
    type CallStack = TTerm;
    type Program = TTerm;

    type State = VmState<Stack, Locals, Memory, CallStack, Program>;
    let formatted =
        format!("{:?}", VmState::<Stack, Locals, Memory, CallStack, Program>(PhantomData));

    assert!(formatted.contains("VmState"));
}
