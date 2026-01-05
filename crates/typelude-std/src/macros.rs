#[macro_export]
macro_rules! define_arith_op {
    ($op_name:ident, $trait:path, $doc:literal) => {
        $crate::paste::paste! {
            // Operator Symbol
            #[doc = $doc]
            pub struct [<Op $op_name>];

            // Expression Struct
            pub struct [<E $op_name>]<Lhs, Rhs>(std::marker::PhantomData<(Lhs, Rhs)>);

            // Updated: Uses $trait (TypeAdd, etc.) instead of typenum traits
            impl<Lhs, Rhs> $crate::typelude_core::Eval for [<E $op_name>]<Lhs, Rhs>
            where
                Lhs: $crate::typelude_core::Eval,
                Rhs: $crate::typelude_core::Eval,
                $crate::typelude_core::Evaluate<Lhs>: $trait<$crate::typelude_core::Evaluate<Rhs>>,
                <$crate::typelude_core::Evaluate<Lhs> as $trait<$crate::typelude_core::Evaluate<Rhs>>>::Output: $crate::typelude_core::Eval,
            {
                type Output = <$crate::typelude_core::Evaluate<Lhs> as $trait<$crate::typelude_core::Evaluate<Rhs>>>::Output;
            }

            // Apply Implementation
            impl<Lhs, Rhs> $crate::typelude_core::Apply<(Lhs, Rhs)> for [<Op $op_name>] {
                type Output = [<E $op_name>]<Lhs, Rhs>;
            }
        }
    };
}

#[macro_export]
macro_rules! define_logic_op {
    ($op_name:ident, $trait:path, $method:ident, $doc:literal) => {
        $crate::paste::paste! {
            // Operator Symbol
            #[doc = $doc]
            pub struct [<Op $op_name>];

            // Expression Struct
            pub struct [<E $op_name>]<Lhs, Rhs>(std::marker::PhantomData<(Lhs, Rhs)>);

            impl<Lhs, Rhs> $crate::typelude_core::Eval for [<E $op_name>]<Lhs, Rhs>
            where
                Lhs: $crate::typelude_core::Eval,
                Rhs: $crate::typelude_core::Eval,
                $crate::typelude_core::Evaluate<Lhs>: $trait,
                $crate::typelude_core::Evaluate<Rhs>: $trait,
            {
                type Output = <$crate::typelude_core::Evaluate<Lhs> as $trait>::$method<$crate::typelude_core::Evaluate<Rhs>>;
            }

            // Apply Implementation
            impl<Lhs, Rhs> $crate::typelude_core::Apply<(Lhs, Rhs)> for [<Op $op_name>] {
                type Output = [<E $op_name>]<Lhs, Rhs>;
            }
        }
    };
}

#[macro_export]
macro_rules! define_unary_logic_op {
    ($op_name:ident, $trait:path, $method:ident, $doc:literal) => {
        $crate::paste::paste! {
            // Operator Symbol
            #[doc = $doc]
            pub struct [<Op $op_name>];

            // Expression Struct
            pub struct [<E $op_name>]<Val>(std::marker::PhantomData<Val>);

            impl<Val> $crate::typelude_core::Eval for [<E $op_name>]<Val>
            where
                Val: $crate::typelude_core::Eval,
                $crate::typelude_core::Evaluate<Val>: $trait,
            {
                type Output = <$crate::typelude_core::Evaluate<Val> as $trait>::$method;
            }

            // Apply Implementation
            impl<Val> $crate::typelude_core::Apply<Val> for [<Op $op_name>] {
                type Output = [<E $op_name>]<Val>;
            }
        }
    };
}

/// Assert that a type-level boolean is true at compile time.
///
/// If the condition evaluates to `TyFalse` (or isn't `TyTrue`), this will trigger a compilation error.
/// The `Condition` must implement `Reify<bool>` (usually via `Eval` -> `TyTrue/TyFalse`).
///
/// # Example
///
/// ```rust,compile_fail
/// use typelude_core::{static_assert_true, std::bool::TyFalse};
///
/// static_assert_true!(TyFalse, "This should fail");
/// ```
#[macro_export]
macro_rules! static_assert_true {
    ($Condition:ty, $Message:literal) => {
        const _: () = {
            // Reify the type to a boolean value
            // We use fully qualified path to ensure we use the correct trait
            use $crate::{eval::Evaluate, std::reify::Reify};

            // Note: $Condition might be an expression needing evaluation
            type Evaluated = Evaluate<$Condition>;

            // Assert const condition
            if !<Evaluated as Reify<bool>>::REIFIED {
                panic!($Message); // This becomes a compile_error in const context
            }
        };
    };
}
