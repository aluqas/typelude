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
            impl<Lhs, Rhs> $crate::model::traits::Apply<(Lhs, Rhs)> for [<Op $op_name>] {
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
            impl<Lhs, Rhs> $crate::model::traits::Apply<(Lhs, Rhs)> for [<Op $op_name>] {
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
            impl<Val> $crate::model::traits::Apply<Val> for [<Op $op_name>] {
                type Output = [<E $op_name>]<Val>;
            }
        }
    };
}
