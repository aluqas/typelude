#[macro_export]
macro_rules! define_arith_op {
    ($op_name:ident, $trait:path, $doc:literal) => {
        $crate::paste::paste! {
            // Operator Symbol
            #[doc = $doc]
            pub struct [<Op $op_name>];
            impl $crate::eval::Sealed for [<Op $op_name>] {}

            // Expression Struct
            pub struct [<E $op_name>]<Lhs, Rhs>(std::marker::PhantomData<(Lhs, Rhs)>);

            // Updated: Uses $trait (TypeAdd, etc.) instead of typenum traits
            impl<Lhs, Rhs> $crate::eval::Eval for [<E $op_name>]<Lhs, Rhs>
            where
                Lhs: $crate::eval::Eval,
                Rhs: $crate::eval::Eval,
                $crate::eval::Evaluate<Lhs>: $trait<$crate::eval::Evaluate<Rhs>>,
                <$crate::eval::Evaluate<Lhs> as $trait<$crate::eval::Evaluate<Rhs>>>::Output: $crate::eval::Eval,
            {
                type Output = <$crate::eval::Evaluate<Lhs> as $trait<$crate::eval::Evaluate<Rhs>>>::Output;
            }

            // Apply Implementation
            impl<Lhs, Rhs> $crate::kernel::traits::Apply<(Lhs, Rhs)> for [<Op $op_name>] {
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
            impl $crate::eval::Sealed for [<Op $op_name>] {}

            // Expression Struct
            pub struct [<E $op_name>]<Lhs, Rhs>(std::marker::PhantomData<(Lhs, Rhs)>);

            impl<Lhs, Rhs> $crate::eval::Eval for [<E $op_name>]<Lhs, Rhs>
            where
                Lhs: $crate::eval::Eval,
                Rhs: $crate::eval::Eval,
                $crate::eval::Evaluate<Lhs>: $trait,
                $crate::eval::Evaluate<Rhs>: $trait,
            {
                type Output = <$crate::eval::Evaluate<Lhs> as $trait>::$method<$crate::eval::Evaluate<Rhs>>;
            }

            // Apply Implementation
            impl<Lhs, Rhs> $crate::kernel::traits::Apply<(Lhs, Rhs)> for [<Op $op_name>] {
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
            impl $crate::eval::Sealed for [<Op $op_name>] {}

            // Expression Struct
            pub struct [<E $op_name>]<Val>(std::marker::PhantomData<Val>);

            impl<Val> $crate::eval::Eval for [<E $op_name>]<Val>
            where
                Val: $crate::eval::Eval,
                $crate::eval::Evaluate<Val>: $trait,
            {
                type Output = <$crate::eval::Evaluate<Val> as $trait>::$method;
            }

            // Apply Implementation
            impl<Val> $crate::kernel::traits::Apply<Val> for [<Op $op_name>] {
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
            use $crate::std::reify::Reify;
            use $crate::eval::Evaluate;

            // Note: $Condition might be an expression needing evaluation
            type Evaluated = Evaluate<$Condition>;

            // Assert const condition
            if !<Evaluated as Reify<bool>>::REIFIED {
                panic!($Message); // This becomes a compile_error in const context
            }
        };
    };
}
