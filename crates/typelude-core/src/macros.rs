//! **Type-Level Macros**
//!
//! Declarative macros for defining type-level operations, AST nodes, and evaluation logic.

// =============================================================================
// Component Macros
// =============================================================================

/// Defines the AST struct definition with PhantomData.
#[macro_export]
macro_rules! def_ast_struct {
    (
        $name:ident,
        ($($arg:ident),*)
    ) => {
        pub struct $name<$($arg),*>(std::marker::PhantomData<($($arg),*)>);
    };
}

/// Implements `Eval` for a given struct.
#[macro_export]
macro_rules! impl_eval {
    (
        $name:ident,
        ($($arg:ident),*),
        where [ $($preds:tt)* ],
        type Output = $out:ty
    ) => {
        impl<$($arg),*> $crate::eval::Eval for $name<$($arg),*>
        where
            $($preds)*
        {
            type Output = $out;
        }
    };
}

/// Defines the Op marker struct and implements Sealed.
#[macro_export]
macro_rules! def_op_marker {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub struct $name;
        impl $crate::eval::Sealed for $name {}
    };
}

/// Implements `Apply` for the Op marker to return the Output type.
#[macro_export]
macro_rules! impl_apply {
    (
        $op:ident,
        ($($arg:ident),*),
        type Output = $out:ty
    ) => {
        impl<$($arg),*> $crate::kernel::traits::Apply<($($arg),*)> for $op {
            type Output = $out;
        }
    };
}

// =============================================================================
// High-Level Interface
// =============================================================================

/// Defines a type-level operation.
///
/// Supports two patterns:
/// 1. **AST Pattern**: Defines a new AST struct and its evaluation logic.
/// 2. **Alias Pattern**: Defines an Op that maps to an existing type/AST.
///
/// # Examples
///
/// ```ignore
/// // AST Pattern
/// def_op! {
///     name: OpAdd,
///     args: (Lhs, Rhs),
///     ast: EAdd {
///         where: [
///             Lhs: Eval,
///             Rhs: Eval,
///             Evaluate<Lhs>: Add<Evaluate<Rhs>>
///         ],
///         type Output = <Evaluate<Lhs> as Add<Evaluate<Rhs>>>::Output
///     }
/// }
///
/// // Alias Pattern
/// def_op! {
///     name: OpInc,
///     args: (N),
///     alias: EAdd<N, P1>
/// }
/// ```
#[macro_export]
macro_rules! def_op {
    // AST Pattern
    (
        $(#[$meta:meta])*
        name: $op:ident,
        args: ($($arg:ident),*),
        ast: $ast:ident {
            where: [ $($preds:tt)* ],
            type Output = $out:ty
        }
    ) => {
        // 1. Define AST Struct
        $crate::def_ast_struct!($ast, ($($arg),*));

        // 2. Implement Eval for AST
        $crate::impl_eval!(
            $ast,
            ($($arg),*),
            where [ $($preds)* ],
            type Output = $out
        );

        // 3. Define Op Marker
        $crate::def_op_marker!(
            $(#[$meta])*
            $op
        );

        // 4. Implement Apply linking Op to AST
        $crate::impl_apply!(
            $op,
            ($($arg),*),
            type Output = $ast<$($arg),*>
        );
    };

    // Alias Pattern
    (
        $(#[$meta:meta])*
        name: $op:ident,
        args: ($($arg:ident),*),
        alias: $alias:ty
    ) => {
        // 1. Define Op Marker
        $crate::def_op_marker!(
            $(#[$meta])*
            $op
        );

        // 2. Implement Apply linking Op to Alias Type
        $crate::impl_apply!(
            $op,
            ($($arg),*),
            type Output = $alias
        );
    };
}
