use typelude_vm::{
    opcode::{
        control::{OpCall, OpIf, OpWhile},
        local::{OpGetLocal, OpSetLocal},
        stack::OpPush,
    },
    vm::value::Lit,
};
use typenum::{U0, U1};

#[test]
fn opcode_debug_names_are_stable() {
    use typelude_col::TTerm;

    let push = format!("{:?}", OpPush::<Lit<U1>>::default());
    let get_local = format!("{:?}", OpGetLocal::<Lit<U0>>::default());
    let set_local = format!("{:?}", OpSetLocal::<Lit<U0>>::default());
    let call = format!("{:?}", OpCall::<()>::default());
    let if_op = format!("{:?}", OpIf::<TTerm, TTerm>::default());
    let while_op = format!("{:?}", OpWhile::<TTerm, TTerm>::default());

    assert!(push.contains("OpPush"));
    assert!(get_local.contains("OpGetLocal"));
    assert!(set_local.contains("OpSetLocal"));
    assert!(call.contains("OpCall"));
    assert!(if_op.contains("OpIf"));
    assert!(while_op.contains("OpWhile"));
}
