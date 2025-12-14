//! **Type-Level Program Macros**
//!
//! Macros for writing type-level stack machine programs in an S-expression style.

/// Maps integer literals to typenum types.
/// Supports 0-100.
#[macro_export]
macro_rules! uint {
    (0) => { $crate::typenum::U0 };
    (1) => { $crate::typenum::U1 };
    (2) => { $crate::typenum::U2 };
    (3) => { $crate::typenum::U3 };
    (4) => { $crate::typenum::U4 };
    (5) => { $crate::typenum::U5 };
    (6) => { $crate::typenum::U6 };
    (7) => { $crate::typenum::U7 };
    (8) => { $crate::typenum::U8 };
    (9) => { $crate::typenum::U9 };
    (10) => { $crate::typenum::U10 };
    (11) => { $crate::typenum::U11 };
    (12) => { $crate::typenum::U12 };
    (13) => { $crate::typenum::U13 };
    (14) => { $crate::typenum::U14 };
    (15) => { $crate::typenum::U15 };
    (16) => { $crate::typenum::U16 };
    (17) => { $crate::typenum::U17 };
    (18) => { $crate::typenum::U18 };
    (19) => { $crate::typenum::U19 };
    (20) => { $crate::typenum::U20 };
    (21) => { $crate::typenum::U21 };
    (22) => { $crate::typenum::U22 };
    (23) => { $crate::typenum::U23 };
    (24) => { $crate::typenum::U24 };
    (25) => { $crate::typenum::U25 };
    (26) => { $crate::typenum::U26 };
    (27) => { $crate::typenum::U27 };
    (28) => { $crate::typenum::U28 };
    (29) => { $crate::typenum::U29 };
    (30) => { $crate::typenum::U30 };
    (31) => { $crate::typenum::U31 };
    (32) => { $crate::typenum::U32 };
    (33) => { $crate::typenum::U33 };
    (34) => { $crate::typenum::U34 };
    (35) => { $crate::typenum::U35 };
    (36) => { $crate::typenum::U36 };
    (37) => { $crate::typenum::U37 };
    (38) => { $crate::typenum::U38 };
    (39) => { $crate::typenum::U39 };
    (40) => { $crate::typenum::U40 };
    (41) => { $crate::typenum::U41 };
    (42) => { $crate::typenum::U42 };
    (43) => { $crate::typenum::U43 };
    (44) => { $crate::typenum::U44 };
    (45) => { $crate::typenum::U45 };
    (46) => { $crate::typenum::U46 };
    (47) => { $crate::typenum::U47 };
    (48) => { $crate::typenum::U48 };
    (49) => { $crate::typenum::U49 };
    (50) => { $crate::typenum::U50 };
    (51) => { $crate::typenum::U51 };
    (52) => { $crate::typenum::U52 };
    (53) => { $crate::typenum::U53 };
    (54) => { $crate::typenum::U54 };
    (55) => { $crate::typenum::U55 };
    (56) => { $crate::typenum::U56 };
    (57) => { $crate::typenum::U57 };
    (58) => { $crate::typenum::U58 };
    (59) => { $crate::typenum::U59 };
    (60) => { $crate::typenum::U60 };
    (61) => { $crate::typenum::U61 };
    (62) => { $crate::typenum::U62 };
    (63) => { $crate::typenum::U63 };
    (64) => { $crate::typenum::U64 };
    (65) => { $crate::typenum::U65 };
    (66) => { $crate::typenum::U66 };
    (67) => { $crate::typenum::U67 };
    (68) => { $crate::typenum::U68 };
    (69) => { $crate::typenum::U69 };
    (70) => { $crate::typenum::U70 };
    (71) => { $crate::typenum::U71 };
    (72) => { $crate::typenum::U72 };
    (73) => { $crate::typenum::U73 };
    (74) => { $crate::typenum::U74 };
    (75) => { $crate::typenum::U75 };
    (76) => { $crate::typenum::U76 };
    (77) => { $crate::typenum::U77 };
    (78) => { $crate::typenum::U78 };
    (79) => { $crate::typenum::U79 };
    (80) => { $crate::typenum::U80 };
    (81) => { $crate::typenum::U81 };
    (82) => { $crate::typenum::U82 };
    (83) => { $crate::typenum::U83 };
    (84) => { $crate::typenum::U84 };
    (85) => { $crate::typenum::U85 };
    (86) => { $crate::typenum::U86 };
    (87) => { $crate::typenum::U87 };
    (88) => { $crate::typenum::U88 };
    (89) => { $crate::typenum::U89 };
    (90) => { $crate::typenum::U90 };
    (91) => { $crate::typenum::U91 };
    (92) => { $crate::typenum::U92 };
    (93) => { $crate::typenum::U93 };
    (94) => { $crate::typenum::U94 };
    (95) => { $crate::typenum::U95 };
    (96) => { $crate::typenum::U96 };
    (97) => { $crate::typenum::U97 };
    (98) => { $crate::typenum::U98 };
    (99) => { $crate::typenum::U99 };
    (100) => { $crate::typenum::U100 };
    // Fallback for non-matching tokens (e.g. types)
    ($other:tt) => { $other };
}

/// Defines variable aliases for memory addresses.
///
/// Example:
/// ```rust,ignore
/// define_vars! { x, y, z }
/// // Expands to:
/// // type x = U0;
/// // type y = U1;
/// // type z = U2;
/// ```
#[macro_export]
macro_rules! define_vars {
    // Entry point
    ( $($vars:ident),* ) => {
        $crate::define_vars_impl!( [ $($vars)* ] [ ] )
    };
}

/// Helper for define_vars
#[macro_export]
macro_rules! define_vars_impl {
    // No more vars
    ( [] [$($cnt:tt)*] ) => {};

    // Match var, assign counter, recurse
    ( [$head:ident $($tail:ident)*] [$($cnt:tt)*] ) => {
        #[allow(non_camel_case_types)]
        pub type $head = $crate::unary_to_uint!($($cnt)*);
        $crate::define_vars_impl!( [$($tail)*] [$($cnt)* I] );
    };
}

/// Helper to convert unary counter (I I I) to typenum
#[macro_export]
macro_rules! unary_to_uint {
    () => { $crate::typenum::U0 };
    (I) => { $crate::typenum::U1 };
    (I I) => { $crate::typenum::U2 };
    (I I I) => { $crate::typenum::U3 };
    (I I I I) => { $crate::typenum::U4 };
    (I I I I I) => { $crate::typenum::U5 };
    (I I I I I I) => { $crate::typenum::U6 };
    (I I I I I I I) => { $crate::typenum::U7 };
    (I I I I I I I I) => { $crate::typenum::U8 };
    (I I I I I I I I I) => { $crate::typenum::U9 };
    (I I I I I I I I I I) => { $crate::typenum::U10 };
    // Expand as needed
}

/// The main macro for writing programs.
///
/// Syntax:
/// ```rust,ignore
/// type Prog = program! {
///     (push 10)
///     (push 1)
///     (add)
/// };
/// ```
#[macro_export]
macro_rules! program {
    ( $($rest:tt)* ) => {
        $crate::parse_body!( [ ] $($rest)* )
    };
}

/// Helper to parse the body of the program.
/// Args: [AccumulatedProg...] RemainingTokens...
#[macro_export]
macro_rules! parse_body {
    // End of program: return the accumulated tyarray
    ( [$($prog:tt)*] ) => {
        $crate::tyarray![ $($prog)* ]
    };

    // --- Instructions ---

    // (push ...)
    ( [$($prog:tt)*] (push $($args:tt)+) $($rest:tt)* ) => {
        $crate::parse_body_push_helper!( [$($prog)*] [ $($args)+ ] $($rest)* )
    };

    // (dup)
    ( [$($prog:tt)*] (dup) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpDup,] $($rest)* )
    };

    // (swap)
    ( [$($prog:tt)*] (swap) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpSwap,] $($rest)* )
    };

    // (drop)
    ( [$($prog:tt)*] (drop) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpDrop,] $($rest)* )
    };

    // (add)
    ( [$($prog:tt)*] (add) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpAdd,] $($rest)* )
    };

    // (sub)
    ( [$($prog:tt)*] (sub) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpSub,] $($rest)* )
    };

    // (eq)
    ( [$($prog:tt)*] (eq) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpEq,] $($rest)* )
    };

    // (neq)
    ( [$($prog:tt)*] (neq) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpNeq,] $($rest)* )
    };

    // (lt)
    ( [$($prog:tt)*] (lt) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpLt,] $($rest)* )
    };

    // (gt)
    ( [$($prog:tt)*] (gt) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpGt,] $($rest)* )
    };

    // (not)
    ( [$($prog:tt)*] (not) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpNot,] $($rest)* )
    };

    // (and)
    ( [$($prog:tt)*] (and) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpAnd,] $($rest)* )
    };

    // (or)
    ( [$($prog:tt)*] (or) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpOr,] $($rest)* )
    };

    // (call Target) - Target is a type
    ( [$($prog:tt)*] (call $target:ty) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpCall<$target>,] $($rest)* )
    };

    // (return)
    ( [$($prog:tt)*] (return) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpReturn,] $($rest)* )
    };

    // --- Control Flow ---

    // (if (Then...) (Else...))
    ( [$($prog:tt)*] (if ($($then:tt)*) ($($else:tt)*)) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpIf<
                    $crate::program!( $($then)* ),
                    $crate::program!( $($else)* )
                >,
            ]
            $($rest)*
        )
    };

    // (while (Cond...) (Body...))
    ( [$($prog:tt)*] (while ($($cond:tt)*) ($($body:tt)*)) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpWhile<
                    $crate::program!( $($cond)* ),
                    $crate::program!( $($body)* )
                >,
            ]
            $($rest)*
        )
    };

    // --- Variable Access (Sugar) ---

    // (load var)
    ( [$($prog:tt)*] (load $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpPush<$v>,
                $crate::machine::instruction::OpLoad,
            ]
            $($rest)*
        )
    };

    // (store var) -> Expects Value on stack. Pushes Addr, Swaps, Stores.
    // [Val] -> [Val, Addr] -> [Addr, Val] -> Store
    ( [$($prog:tt)*] (store $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpPush<$v>,
                $crate::machine::instruction::OpSwap,
                $crate::machine::instruction::OpStore,
            ]
            $($rest)*
        )
    };
}

/// Helper to decide if (push X) is a literal or a path/ident.
#[macro_export]
macro_rules! parse_body_push_helper {
    // Single token: try uint! with fallback
    ( [$($prog:tt)*] [ $n:tt ] $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpPush< $crate::uint!($n) >, ]
            $($rest)*
        )
    };

    // Multi-token: assume type
    ( [$($prog:tt)*] [ $($t:tt)+ ] $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpPush< $($t)+ >, ]
            $($rest)*
        )
    };
}

#[cfg(test)]
mod tests {
    use crate::eval::{Evaluator};
    use crate::machine::execution::ERun;
    use crate::machine::state::MachineState;
    use crate::types::array::{TyNil, TyArray};
    use typenum::{U0, U4};
    use static_assertions::assert_type_eq_all;

    // Helper trait to extract stack from MachineState
    trait GetStack {
        type Output;
    }
    impl<S, M, C, P> GetStack for MachineState<S, M, C, P> {
        type Output = S;
    }

    #[test]
    fn test_simple_program() {
        type Prog = crate::program! {
            (push 3)
            (push 1)
            (add)
        };
        type InitialState = MachineState<TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        assert_type_eq_all!(FinalStack, TyArray<U4, TyNil>);
    }

    #[test]
    fn test_control_flow() {
        // if 3 > 1 { push 10 } else { push 20 }
        type Prog = crate::program! {
            (push 3)
            (push 1)
            (gt)
            (if ((push 10)) ((push 20)))
        };
        type InitialState = MachineState<TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        assert_type_eq_all!(FinalStack, TyArray<crate::typenum::U10, TyNil>);
    }

    #[test]
    fn test_while_loop() {
        // While top > 0, sub 1
        // Start with 3
        type Prog = crate::program! {
            (push 3)
            (while ((dup) (push 0) (gt)) ((push 1) (sub)))
        };
        type InitialState = MachineState<TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        assert_type_eq_all!(FinalStack, TyArray<U0, TyNil>);
    }

    #[test]
    fn test_variables() {
        // Define vars
        define_vars! { x, y }
        // x=0, y=1

        // Mem[x] = 10
        // Mem[y] = 5
        // Push Mem[x] + Mem[y]
        type Prog = crate::program! {
            (push 10)
            (store x)
            (push 5)
            (store y)
            (load x)
            (load y)
            (add)
        };

        // Need to initialize memory with enough zeros for x and y
        type InitialMemory = crate::tyarray![U0, U0];
        type InitialState = MachineState<TyNil, InitialMemory, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        assert_type_eq_all!(FinalStack, TyArray<crate::typenum::U15, TyNil>);
    }
}
