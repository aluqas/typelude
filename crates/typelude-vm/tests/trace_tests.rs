use typelude_std::std::debug::trace::Trace;
use typelude_vm::machine::instruction::{OpCall, OpGetLocal, OpIf, OpPush, OpSetLocal, OpWhile};
use typenum::{U0, U1};

#[test]
fn test_vm_instruction_trace() {
    assert_eq!(OpPush::<U1>::fmt(), "Push(1)");
    assert_eq!(OpGetLocal::<U0>::fmt(), "GetLocal(0)");
    assert_eq!(OpSetLocal::<U0>::fmt(), "SetLocal(0)");

    // Check static strings for control flow placeholders
    use typelude_std::std::col::array::Nil;
    assert_eq!(OpCall::<()>::fmt(), "Call");
    assert_eq!(OpIf::<Nil, Nil>::fmt(), "If(...)");
    assert_eq!(OpWhile::<Nil, Nil>::fmt(), "While(...)");
}
