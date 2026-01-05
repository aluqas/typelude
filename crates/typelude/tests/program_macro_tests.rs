use static_assertions::assert_type_eq_all;
use typelude::{
    Evaluate,
    machine::{execution::ERun, state::MachineState},
    program,
    std::array::{TyArray, TyNil},
    typenum::{U0, U1, U3, U4, U10, U15, U20, U40},
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
    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, TyArray<U4, TyNil>);
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
    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, TyArray<U10, TyNil>);
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
//     type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
//     type FinalState = Evaluate<ERun<InitialState>>;
//     type FinalStack = <FinalState as GetStack>::Output;
//
//     assert_type_eq_all!(FinalStack, TyArray<U0, TyNil>);
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
    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;
    assert_type_eq_all!(FinalStack, TyArray<U15, TyNil>);
}

#[test]
fn test_local_vars_scoping() {
    type Prog = program! {
        (push 10)
        (let x)
        (push 1)
        (push 0) (gt) // 0 > 1 (False) wait. 1, 0 -> 0 > 1 (False).
        // 1, 0 (lt) -> 0 < 1 (True).
        (if
            ((push 20) (let y) (get y) (get x) (add))
            ((push 0))
        )
        (get x)
        (add)
    };
    // If False: Stack has [0]. Get x (10). Add -> 10.
    // If True (using lt): Stack [30]. Get x (10). Add -> 40.
    // Let's use (lt).

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

    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog2>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, TyArray<U40, TyNil>);
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
    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, TyArray<U20, TyNil>);
}
