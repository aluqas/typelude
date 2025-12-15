/// Defines variable aliases for memory addresses.
#[macro_export]
macro_rules! define_vars {
    ( $($vars:ident),* ) => {
        // Generate types (Legacy/Global)
        $crate::define_vars_impl!( [ $($vars)* ] [ ] );

        // Generate identifier mapping macro for local variable equality check
        // Pass '$' token to avoid nesting issues
        $crate::define_vars_mapper_accum!( __typelude_var_mapper, [ $($vars)* ], [], [], $ );
    };
}

#[macro_export]
macro_rules! define_vars_impl {
    ( [] [$($cnt:tt)*] ) => {};
    ( [$head:ident $($tail:ident)*] [$($cnt:tt)*] ) => {
        #[allow(non_camel_case_types)]
        pub type $head = $crate::unary_to_uint!($($cnt)*);
        $crate::define_vars_impl!( [$($tail)*] [$($cnt)* I] );
    };
}

/// Helper to generate the `__typelude_var_mapper` macro.
/// Args: Name, Vars, Counter, RulesAccumulator, DollarToken
#[macro_export]
macro_rules! define_vars_mapper_accum {
    // Base case: No more vars. Generate the macro definition.
    ( $name:ident, [], [$($cnt:tt)*], [$($rules:tt)*], $d:tt ) => {
         #[allow(unused_macros)]
         macro_rules! $name {
             $($rules)*
             // Fallback: If variable unknown, we can't do anything.
         }
    };

    // Recursive step: Add a rule for $head
    // The rule format is CPS: ($head, [$cb_path...] ! ( $args... )) => { $cb_path!( $args... [ID] ) }
    // We expect the callback path to be wrapped in brackets [ ... ] to avoid 'path' fragment ambiguity.
    ( $name:ident, [$head:ident $($tail:ident)*], [$($cnt:tt)*], [$($rules:tt)*], $d:tt ) => {
        $crate::define_vars_mapper_accum!(
            $name,
            [$($tail)*],
            [$($cnt)* I], // Increment counter
            [
                $($rules)*
                ($head, [ $d($d cb:tt)* ] ! ( $d($d args:tt)* )) => {
                    $d($d cb)* ! ( $d($d args)* [ $($cnt)* ] )
                };
            ],
            $d
        );
    };
}

#[macro_export]
macro_rules! unary_to_uint {
    () => {
        $crate::typenum::U0
    };
    (I) => {
        $crate::typenum::U1
    };
    (I I) => {
        $crate::typenum::U2
    };
    (I I I) => {
        $crate::typenum::U3
    };
    (I I I I) => {
        $crate::typenum::U4
    };
    (I I I I I) => {
        $crate::typenum::U5
    };
    (I I I I I I) => {
        $crate::typenum::U6
    };
    (I I I I I I I) => {
        $crate::typenum::U7
    };
    (I I I I I I I I) => {
        $crate::typenum::U8
    };
    (I I I I I I I I I) => {
        $crate::typenum::U9
    };
    (I I I I I I I I I I) => {
        $crate::typenum::U10
    };
}

// --- Local Variable Lookup Helper ---

/// Finds the index of a variable in the local variable list.
/// Format: `find_var_index!(TargetVar, [HeadVar TailVars...], IndexAccumulator)`
/// Returns: typenum::Unsigned type (U0, U1, ...)
#[macro_export]
macro_rules! find_var_index {
    // Found at head? We need to compare TargetVar and HeadVar using mapper
    ($target:ident, [$head:ident $($tail:ident)*], [$($acc:tt)*]) => {
         $crate::check_vars_eq!($target, $head,
            // Equal: Return index
            [ $crate::unary_to_uint!($($acc)*) ],
            // Not Equal: Recurse
            [ $crate::find_var_index!($target, [$($tail)*], [$($acc)* I]) ]
         )
    };
    // Not found (Empty list)
    ($target:ident, [], [$($acc:tt)*]) => {
        compile_error!(concat!("Variable not found in scope: ", stringify!($target)))
    };
}

// CPS helpers for variable equality check

/// Checks if v1 == v2.
/// Usage: `check_vars_eq!(v1, v2, [then_block], [else_block])`
#[macro_export]
macro_rules! check_vars_eq {
    ($v1:ident, $v2:ident, [$($then:tt)*], [$($else:tt)*]) => {
        // Resolve v1. Pass (v2, then, else) as args to callback.
        // Wrap callback in [ ... ]
        __typelude_var_mapper!( $v1, [ $crate::check_vars_eq_step2 ] ! ( $v2, [$($then)*], [$($else)*], ) )
    };
}

#[macro_export]
macro_rules! check_vars_eq_step2 {
    // Callback from v1 resolution. Args: v2, then, else, [ID1]
    ( $v2:ident, [$($then:tt)*], [$($else:tt)*], [$($id1:tt)*] ) => {
        // Resolve v2. Pass (ID1, then, else) as args.
        __typelude_var_mapper!( $v2, [ $crate::check_vars_eq_step3 ] ! ( [$($id1)*], [$($then)*], [$($else)*], ) )
    };
}

#[macro_export]
macro_rules! check_vars_eq_step3 {
    // Callback from v2 resolution. Args: ID1, then, else, [ID2]
    ( [$($id1:tt)*], [$($then:tt)*], [$($else:tt)*], [$($id2:tt)*] ) => {
        $crate::check_unary_eq!( [$($id1)*], [$($id2)*], [$($then)*], [$($else)*] )
    };
}

/// Checks if two unary sequences are equal
#[macro_export]
macro_rules! check_unary_eq {
    ( [], [], [$($then:tt)*], [$($else:tt)*] ) => { $($then)* };
    ( [I $($rest1:tt)*], [I $($rest2:tt)*], [$($then:tt)*], [$($else:tt)*] ) => {
        $crate::check_unary_eq!( [$($rest1)*], [$($rest2)*], [$($then)*], [$($else)*] )
    };
    // Mismatch
    ( [$($a:tt)*], [$($b:tt)*], [$($then:tt)*], [$($else:tt)*] ) => { $($else)* };
}

// **Type-Level Program Macros**
//
// Macros for writing type-level stack machine programs in an S-expression style.

/// Maps integer literals to typenum types using const-generics.
/// Supports any integer that fits in strict `typenum::Const<N>`.
#[macro_export]
macro_rules! uint {
    ($n:literal) => {
        <$crate::typenum::Const<$n> as $crate::typenum::ToUInt>::Output
    };
}

// --- Main Program Macro ---

/// The main macro for writing programs.
///
/// Syntax:
/// ```rust,ignore
/// type Prog = program! {
///     (push 10)
///     (let x)
///     (get x)
///     (add)
/// };
/// ```
#[macro_export]
macro_rules! program {
    ( $($rest:tt)* ) => {
        // Initial state: Empty Prog, Empty Cleanup, Empty Vars
        $crate::parse_body!( [ ] [ ] [ ] $($rest)* )
    };
}

/// Helper to parse the body of the program.
/// Args: [AccumulatedProg...] [CleanupOps...] [Vars...] RemainingTokens...
#[macro_export]
macro_rules! parse_body {
    // End of program: return the accumulated tyarray
    // Note: We MUST include cleanup ops here to clean up top-level locals!
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] ) => {
        $crate::tyarray![ $($prog)* $($cleanup)* ]
    };

    // --- Instructions ---

    // (push ...)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (push $($args:tt)+) $($rest:tt)* ) => {
        $crate::parse_body_push_helper!( [$($prog)*] [$($cleanup)*] [$($vars)*] [ $($args)+ ] $($rest)* )
    };

    // (dup)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (dup) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpDup,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (swap)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (swap) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpSwap,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (drop)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (drop) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpDrop,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (add)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (add) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpAdd,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (sub)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (sub) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpSub,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (eq)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (eq) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpEq,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (neq)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (neq) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpNeq,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (lt)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (lt) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpLt,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (gt)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (gt) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpGt,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (not)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (not) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpNot,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (and)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (and) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpAnd,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (or)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (or) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpOr,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (call Target)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (call $target:ty) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpCall<$target>,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // (return)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (return) $($rest:tt)* ) => {
        $crate::parse_body!( [$($prog)* $crate::machine::instruction::OpReturn,] [$($cleanup)*] [$($vars)*] $($rest)* )
    };

    // --- Local Variables ---

    // (let var)
    // Adds variable to Vars (Prepend), adds OpLet to Prog, adds OpDropLocal to Cleanup
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (let $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpLet,]
            [$($cleanup)* $crate::machine::instruction::OpDropLocal,]
            [$v $($vars)*]
            $($rest)*
        )
    };

    // (get var)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (get $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpGetLocal< $crate::find_var_index!($v, [$($vars)*], []) >, ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // (set var)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (set $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpSetLocal< $crate::find_var_index!($v, [$($vars)*], []) >, ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // --- Global Variables (Legacy/Global Memory) ---

    // (load var)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (load $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpPush<$v>,
                $crate::machine::instruction::OpLoad,
            ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // (store var)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (store $v:ident) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpPush<$v>,
                $crate::machine::instruction::OpSwap,
                $crate::machine::instruction::OpStore,
            ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // --- Control Flow (Blocks) ---

    // (if (Then...) (Else...))
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (if ($($then:tt)*) ($($else:tt)*)) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpIf<
                    $crate::parse_block!( [$($vars)*] $($then)* ),
                    $crate::parse_block!( [$($vars)*] $($else)* )
                >,
            ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // (while (Cond...) (Body...))
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] (while ($($cond:tt)*) ($($body:tt)*)) $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)*
                $crate::machine::instruction::OpWhile<
                    $crate::parse_block!( [$($vars)*] $($cond)* ),
                    $crate::parse_block!( [$($vars)*] $($body)* )
                >,
            ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };
}

/// Helper to parse a block.
/// It starts with inherited Vars, but empty Prog and empty Cleanup.
/// At the end, it appends the Cleanup instructions to the Prog.
/// Args: [Vars...] Tokens...
#[macro_export]
macro_rules! parse_block {
    ( [$($vars:ident)*] $($tokens:tt)* ) => {
        $crate::parse_block_impl!( [ ] [ ] [$($vars)*] $($tokens)* )
    };
}

/// Implementation of parse_block. Uses same logic as parse_body but finishes differently.
#[macro_export]
macro_rules! parse_block_impl {
    // End of block: return Prog + Cleanup
    // Cleanup instructions are OpDropLocal.
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] ) => {
        $crate::tyarray![ $($prog)* $($cleanup)* ]
    };

    // Recursion to parse_body (same as before)
    ( $($rest:tt)* ) => {
        $crate::parse_body!( $($rest)* )
    };
}

/// Helper to decide if (push X) is a literal or a path/ident.
#[macro_export]
macro_rules! parse_body_push_helper {
    // Single token: try uint! with fallback
    // We try to use uint!($n) first. If $n is a literal, it works.
    // If $n is a type (U10), uint! might fail if not handled carefully,
    // BUT since we removed the manual uint! which failed on types,
    // the new uint! relies on Const<$n>. Const<Type> fails.
    //
    // However, macro_rules matching order:
    // We can differentiate `literal` from `ident` or `path`?
    // Rust macros match `literal` specifically.

    // Case 1: Literal (e.g. 10)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] [ $n:literal ] $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpPush< $crate::uint!($n) >, ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // Case 2: Type/Ident/Path (e.g. U10, MyType)
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] [ $n:tt ] $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpPush< $n >, ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };

    // Multi-token: assume type
    ( [$($prog:tt)*] [$($cleanup:tt)*] [$($vars:ident)*] [ $($t:tt)+ ] $($rest:tt)* ) => {
        $crate::parse_body!(
            [$($prog)* $crate::machine::instruction::OpPush< $($t)+ >, ]
            [$($cleanup)*]
            [$($vars)*]
            $($rest)*
        )
    };
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::{
        eval::Evaluator,
        machine::{execution::ERun, state::MachineState},
        std::array::{TyArray, TyNil},
    };

    // Helper trait to extract stack from MachineState
    trait GetStack {
        type Output;
    }
    impl<S, L, M, C, P> GetStack for MachineState<S, L, M, C, P> {
        type Output = S;
    }

    #[test]
    fn test_simple_program() {
        use crate::typenum::U4;
        type Prog = crate::program! {
            (push 3)
            (push 1)
            (add)
        };
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
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
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
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
        use crate::typenum::U0;
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        assert_type_eq_all!(FinalStack, TyArray<U0, TyNil>);
    }

    #[test]
    fn test_variables() {
        // Define vars
        use crate::typenum::U0;
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
        type InitialMemory = crate::tyarray![crate::typenum::U0, crate::typenum::U0];
        type InitialState = MachineState<TyNil, TyNil, InitialMemory, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;

        assert_type_eq_all!(FinalStack, TyArray<crate::typenum::U15, TyNil>);
    }

    #[test]
    fn test_local_vars() {
        use crate::typenum::U15; // 10 + 5
        define_vars! { x, y } // Required for local var naming now

        // (push 10)
        // (let x) -> locals: [x=10]
        // (push 5)
        // (let y) -> locals: [y=5, x=10]
        // (get x) -> push 10
        // (get y) -> push 5
        // (add) -> push 15
        // End of scope: drop y, drop x.
        type Prog = crate::program! {
            (push 10)
            (let x)
            (push 5)
            (let y)
            (get x)
            (get y)
            (add)
        };
        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type FinalStack = <FinalState as GetStack>::Output;
        // Result stack should be [15]
        assert_type_eq_all!(FinalStack, TyArray<crate::typenum::U15, TyNil>);

        type ExpectedState =
            MachineState<TyArray<crate::typenum::U15, TyNil>, TyNil, TyNil, TyNil, TyNil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_local_vars_scoping() {
        use crate::typenum::U40;
        define_vars! { x, y }

        // (let x)
        // (if (...) ((let y) (get y)) (...))
        // (get x)
        type Prog = crate::program! {
            (push 10)
            (let x)
            (push 1) // true condition: OpGt check logic? OpIf uses boolean.
            (push 0) (gt) // 1 > 0 is True
            (if
                ((push 20) (let y) (get y) (get x) (add)) // then: let y=20. stack: [30] (20+10)
                ((push 0)) // else
            )
            // After if, y is gone. x is still here.
            // Stack has [30]
            (get x) // 10
            (add) // 40
        };

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;

        type ExpectedState =
            MachineState<TyArray<crate::typenum::U40, TyNil>, TyNil, TyNil, TyNil, TyNil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }

    #[test]
    fn test_shadowing() {
        define_vars! { x }

        // (push 10) (let x)
        // (push 20) (let x) -> Shadows outer x
        // (get x) -> Should be 20
        // drop inner x
        // (get x) -> Should be 10

        type Prog = crate::program! {
            (push 10)
            (let x)
            (push 20)
            (let x)
            (get x) // 20
        };
        // After program: stack [20]. Cleanup drops both.

        type InitialState = MachineState<TyNil, TyNil, TyNil, TyNil, Prog>;
        type FinalState = Evaluator<ERun<InitialState>>;
        type ExpectedState =
            MachineState<TyArray<crate::typenum::U20, TyNil>, TyNil, TyNil, TyNil, TyNil>;
        assert_type_eq_all!(FinalState, ExpectedState);
    }
}
