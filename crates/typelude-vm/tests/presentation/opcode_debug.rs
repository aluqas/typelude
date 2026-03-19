use typelude_vm::opcode::{OpCall, OpGetLocal, OpIf, OpPush, OpSetLocal, OpWhile};
use typenum::{U0, U1};

#[test]
fn opcode_debug_names_are_stable() {
    use typelude_std::std::col::array::Nil;

    let push = format!("{:?}", OpPush::<U1>::default());
    let get_local = format!("{:?}", OpGetLocal::<U0>::default());
    let set_local = format!("{:?}", OpSetLocal::<U0>::default());
    let call = format!("{:?}", OpCall::<()>::default());
    let if_op = format!("{:?}", OpIf::<Nil, Nil>::default());
    let while_op = format!("{:?}", OpWhile::<Nil, Nil>::default());

    assert!(push.contains("OpPush"));
    assert!(get_local.contains("OpGetLocal"));
    assert!(set_local.contains("OpSetLocal"));
    assert!(call.contains("OpCall"));
    assert!(if_op.contains("OpIf"));
    assert!(while_op.contains("OpWhile"));
}
