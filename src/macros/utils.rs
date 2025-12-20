//
// Local Variable Lookup Helper
//

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

//
// Type-Level Program Macros
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
