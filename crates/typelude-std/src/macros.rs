#[macro_export]
macro_rules! define_arith_op {
    ($op_name:ident, $trait:path, $doc:literal) => {
        $crate::paste::paste! {
            $crate::typelude_macros::ty_fn! {
                #[doc = $doc]
                pub struct [<E $op_name>]<Lhs, Rhs>
                where
                    Lhs, Rhs,
                    ~Lhs: $trait<~Rhs>
                {
                    type Output = <~Lhs as $trait<~Rhs>>::Output;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_logic_op {
    ($op_name:ident, $trait:path, $method:ident, $doc:literal) => {
        $crate::paste::paste! {
            $crate::typelude_macros::ty_fn! {
                #[doc = $doc]
                pub struct [<E $op_name>]<Lhs, Rhs>
                where
                    Lhs, Rhs,
                    ~Lhs: $trait,
                    ~Rhs: $trait
                {
                    type Output = <~Lhs as $trait>::$method<~Rhs>;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_unary_logic_op {
    ($op_name:ident, $trait:path, $method:ident, $doc:literal) => {
        $crate::paste::paste! {
            $crate::typelude_macros::ty_fn! {
                #[doc = $doc]
                pub struct [<E $op_name>]<Val>
                where
                    Val,
                    ~Val: $trait
                {
                    type Output = <~Val as $trait>::$method;
                }
            }
        }
    };
}

/// Assert that a type-level boolean is true at compile time.
///
/// If the condition evaluates to `TyFalse` (or isn't `TyTrue`), this will
/// trigger a compilation error. The `Condition` must implement `Reify<bool>`
/// (usually via `Eval` -> `TyTrue/TyFalse`).
///
/// # Example
///
/// ```rust,compile_fail
/// use typelude_std::core::{static_assert_true, std::bool::TyFalse};
///
/// static_assert_true!(TyFalse, "This should fail");
/// ```
#[macro_export]
macro_rules! static_assert_true {
    ($Condition:ty, $Message:literal) => {
        const _: () = {
            // Reify the type to a boolean value
            // We use fully qualified path to ensure we use the correct trait
            use $crate::{core::Evaluate, std::reify::Reify};

            // Note: $Condition might be an expression needing evaluation
            type Evaluated = Evaluate<$Condition>;

            // Assert const condition
            if !<Evaluated as Reify<bool>>::REIFIED {
                panic!($Message); // This becomes a compile_error in const context
            }
        };
    };
}
