#![recursion_limit = "256"]
use seq_macro::seq;
use typelude::{prelude::*, typenum::Unsigned};

// Helper to assert value equality at runtime for the resulting types.
macro_rules! check_add {
    ($n:literal, $m:literal) => {{
        type N = typelude::uint!($n);
        type M = typelude::uint!($m);
        // Result of adding two Unsigned is Unsigned
        type Res = <EAdd<N, M> as Evaluable>::Output;
        assert_eq!(Res::to_u64(), $n + $m, "Add mismatch for {} + {}", $n, $m);
    }};
}

/*
macro_rules! check_sub_unsigned {
    ($n:literal, $m:literal) => {{
        type N = typelude::uint!($n);
        type M = typelude::uint!($m);
        // Result of subtracting Unsigned (where N >= M) is Unsigned
        type Res = <ESub<N, M> as Evaluable>::Output;
        assert_eq!(Res::to_u64(), $n - $m, "Sub mismatch for {} - {}", $n, $m);
    }};
}
*/

#[test]
fn test_exhaustive_add() {
    seq!(N in 0..8 {
        seq!(M in 0..8 {
            check_add!(N, M);
        });
    });
}

/*
#[test]
fn test_exhaustive_sub() {
     seq!(N in 0..8 {
        seq!(M in 0..8 {
             // typenum Unsigned subtraction is only defined for N >= M
             if N >= M {
                 check_sub_unsigned!(N, M);
             }
        });
    });
}
*/
