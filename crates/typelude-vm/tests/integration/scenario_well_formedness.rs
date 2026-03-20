use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::col::array::Nil, tyarray};
use typelude_vm::{
    opcode::{
        local::OpGetLocal,
        numeric::OpAdd,
        stack::{OpDup, OpPush},
    },
    vm::{
        protocol::trap::{BadLocalIndex, StackUnderflow},
        semantics::state::VmState,
        well_formed::{IllFormed, InstrWellFormed, ProgramWellFormed, WellFormed},
    },
};
use typenum::{U0, U1, U2};

#[test]
fn safe_program_has_well_formed_proof() {
    type Program = tyarray![OpPush<ELit<U1>>, OpDup, OpAdd];
    type Initial = VmState<Nil, Nil, Nil, Nil, Program>;
    type Final = VmState<tyarray![ELit<U2>], Nil, Nil, Nil, Nil>;
    type Proof = <Program as ProgramWellFormed<Initial, Program>>::Output;

    assert_type_eq_all!(Proof, WellFormed<Final>);
}

#[test]
fn underflow_program_is_ill_formed() {
    type Program = tyarray![OpAdd];
    type Initial = VmState<Nil, Nil, Nil, Nil, Program>;
    type Proof = <Program as ProgramWellFormed<Initial, Program>>::Output;

    assert_type_eq_all!(Proof, IllFormed<StackUnderflow, Initial>);
}

#[test]
fn invalid_local_access_is_ill_formed() {
    type Program = tyarray![OpGetLocal<ELit<U0>>];
    type Initial = VmState<Nil, Nil, Nil, Nil, Program>;
    type Proof = <OpGetLocal<ELit<U0>> as InstrWellFormed<Initial>>::Output;

    assert_type_eq_all!(Proof, IllFormed<BadLocalIndex<U0>, Initial>);
}
