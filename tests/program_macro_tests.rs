use static_assertions::assert_type_eq_all;
use typelude::{
    Evaluate,
    machine::{execution::ERun, state::MachineState},
    program,
    std::array::{Array, Nil},
    typenum::{U4, U10, U15, U20, U40},
};

trait GetStack {
    type Output;
}
impl<S, L, M, C, P> GetStack for MachineState<S, L, M, C, P> {
    type Output = S;
}

#[test]
fn test_simple_program() {
    type Prog = program! {
        (push 3)
        (push 1)
        (add)
    };
    type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, Array<U4, Nil>);
}

#[test]
fn test_control_flow() {
    // if 3 > 1 { push 10 } else { push 20 }
    // (push 3) (push 1) (gt) -> 1 > 3 (False).
    // (lt) -> 1 < 3 (True).
    type Prog = program! {
        (push 3)
        (push 1)
        (lt)
        (if ((push 10)) ((push 20)))
    };
    type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, Array<U10, Nil>);
}

// TODO: Fix OpWhile execution - currently blocking compilation
// #[test]
// fn test_while_loop() {
//     // While top > 0, sub 1
//     // Start with 3
//     // Cond: Dup, Push 0, Lt -> 0 < Top.
//     type Prog = program! {
//         (push 3)
//         (while ((dup) (push 0) (lt)) ((push 1) (sub)))
//     };
//     type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
//     type FinalState = Evaluate<ERun<InitialState>>;
//     type FinalStack = <FinalState as GetStack>::Output;
//
//     assert_type_eq_all!(FinalStack, Array<U0, Nil>);
// }

#[test]
fn test_local_vars() {
    type Prog = program! {
        (push 10)
        (let x)
        (push 5)
        (let y)
        (get x)
        (get y)
        (add)
    };
    type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;
    assert_type_eq_all!(FinalStack, Array<U15, Nil>);
}

#[test]
fn test_local_vars_scoping() {
    // (push 1) (push 0) (lt) -> 0 < 1 -> True.
    type Prog2 = program! {
        (push 10)
        (let x)
        (push 1)
        (push 0) (lt)
        (if
            ((push 20) (let y) (get y) (get x) (add))
            ((push 0))
        )
        (get x)
        (add)
    };

    type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog2>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, Array<U40, Nil>);
}

#[test]
fn test_shadowing() {
    type Prog = program! {
        (push 10)
        (let x)
        (push 20)
        (let x)
        (get x) // 20
    };
    type InitialState = MachineState<Nil, Nil, Nil, Nil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, Array<U20, Nil>);
}
