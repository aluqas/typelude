use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
use typelude_vm::{
    opcode::{
        local::OpGetLocal,
        numeric::OpAdd,
        stack::{OpDup, OpPush},
    },
    vm::{
        protocol::trap::{BadLocalIndex, StackUnderflow},
        semantics::state::VmState,
        value::Lit,
        well_formed::{IllFormed, InstrWellFormed, ProgramWellFormed, WellFormed},
    },
};
use typenum::{U0, U1, U2};

#[test]
fn safe_program_has_well_formed_proof() {
    type Program = tarr![OpPush<Lit<U1>>, OpDup, OpAdd];
    type Initial = VmState<TTerm, TTerm, TTerm, TTerm, Program>;
    type Final = VmState<tarr![Lit<U2>], TTerm, TTerm, TTerm, TTerm>;
    type Proof = <Program as ProgramWellFormed<Initial, Program>>::Output;

    assert_type_eq_all!(Proof, WellFormed<Final>);
}

#[test]
fn underflow_program_is_ill_formed() {
    type Program = tarr![OpAdd];
    type Initial = VmState<TTerm, TTerm, TTerm, TTerm, Program>;
    type Proof = <Program as ProgramWellFormed<Initial, Program>>::Output;

    assert_type_eq_all!(Proof, IllFormed<StackUnderflow, Initial>);
}

#[test]
fn invalid_local_access_is_ill_formed() {
    type Program = tarr![OpGetLocal<Lit<U0>>];
    type Initial = VmState<TTerm, TTerm, TTerm, TTerm, Program>;
    type Proof = <OpGetLocal<Lit<U0>> as InstrWellFormed<Initial>>::Output;

    assert_type_eq_all!(Proof, IllFormed<BadLocalIndex<U0>, Initial>);
}
