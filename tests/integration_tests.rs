#![recursion_limit = "1024"]
use static_assertions::assert_type_eq_all;
use typelude::{
    define_vars,
    eval::Evaluate,
    machine::{execution::ERun, state::MachineState},
    program,
    std::array::{TyArray, TyNil},
};
use typenum::{U21, U55};

// Helper to extract stack
trait GetStack {
    type Output;
}
impl<S, L, M, C, P> GetStack for MachineState<S, L, M, C, P> {
    type Output = S;
}

// Define common variables for tests
define_vars! { a, b, c, sum, n, res }

#[test]
fn test_fibonacci_iterative() {
    // Fibonacci: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55
    // Calculate 10th fib (55)
    // N=10
    // a=0, b=1
    // while N > 0:
    //   temp = a + b
    //   a = b
    //   b = temp
    //   N = N - 1

    type Prog = program! {
        (push 10) (let n)
        (push 0) (let a)
        (push 1) (let b)

        (while
            // Cond: n > 0
            ((get n) (push 0) (gt))
            // Body
            (
                (get a) (get b) (add) // temp = a + b. Stack: [temp]
                (get b) (set a)       // a = b. Stack: [temp]
                (set b)               // b = temp. Stack: []
                (get n) (push 1) (sub) (set n) // n = n - 1
            )
        )
        (get a) // Result
    };

    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    // The 10th fibonacci number (starting 0, 1, 1, 2...)
    // F0=0, F1=1, F2=1, F3=2, F4=3, F5=5, F6=8, F7=13, F8=21, F9=34, F10=55
    assert_type_eq_all!(FinalStack, TyArray<U55, TyNil>);
}

#[test]
fn test_sum_list() {
    // Sum numbers from 1 to 6 using a loop
    // Result 21
    type Prog = program! {
        (push 0) (let sum)
        (push 6) (let n)
        (while ((get n) (push 0) (gt)) (
            (get sum) (get n) (add) (set sum)
            (get n) (push 1) (sub) (set n)
        ))
        (get sum)
    };
    type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
    type FinalState = Evaluate<ERun<InitialState>>;
    type FinalStack = <FinalState as GetStack>::Output;

    assert_type_eq_all!(FinalStack, TyArray<U21, TyNil>);
}
