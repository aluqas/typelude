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
        $crate::define_vars_mapper_accum! {
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
        }
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
