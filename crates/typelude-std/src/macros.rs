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

#[macro_export]
macro_rules! helper_if {
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident < $($gen:ident),+ >;
        on $true_ty:ty $(where [ $($wtrue:tt)* ])? => $true_out:ty;
        on $false_ty:ty $(where [ $($wfalse:tt)* ])? => $false_out:ty;
    ) => {
        $(#[$meta])*
        $vis trait $name < $($gen),+ > {
            type Output;
        }

        impl< $($gen),+ > $name < $($gen),+ > for $true_ty
        $( where $($wtrue)* )?
        {
            type Output = $true_out;
        }

        impl< $($gen),+ > $name < $($gen),+ > for $false_ty
        $( where $($wfalse)* )?
        {
            type Output = $false_out;
        }
    };
}

#[macro_export]
macro_rules! helper_bit {
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident < $($gen:ident),+ ; $bit:ident > for $self_ty:ty;
        on B1 $(where [ $($w1:tt)* ])? => $out1:ty;
        on B0 $(where [ $($w0:tt)* ])? => $out0:ty;
    ) => {
        $(#[$meta])*
        $vis trait $name < $($gen),+ , $bit > {
            type Output;
        }

        impl< $($gen),+ > $name < $($gen),+ , typenum::B1 > for $self_ty
        $( where $($w1)* )?
        {
            type Output = $out1;
        }

        impl< $($gen),+ > $name < $($gen),+ , typenum::B0 > for $self_ty
        $( where $($w0)* )?
        {
            type Output = $out0;
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident < $($gen:ident),+ ; $bit1:ident, $bit2:ident > for $self_ty:ty;
        on (B1, $b2:ident) $(where [ $($w1:tt)* ])? => $out1:ty;
        on (B0, B1) $(where [ $($w2:tt)* ])? => $out2:ty;
        on (B0, B0) $(where [ $($w3:tt)* ])? => $out3:ty;
    ) => {
        $(#[$meta])*
        $vis trait $name < $($gen),+ , $bit1, $bit2 > {
            type Output;
        }

        impl< $($gen),+ , $b2 > $name < $($gen),+ , typenum::B1, $b2 > for $self_ty
        $( where $($w1)* )?
        {
            type Output = $out1;
        }

        impl< $($gen),+ > $name < $($gen),+ , typenum::B0, typenum::B1 > for $self_ty
        $( where $($w2)* )?
        {
            type Output = $out2;
        }

        impl< $($gen),+ > $name < $($gen),+ , typenum::B0, typenum::B0 > for $self_ty
        $( where $($w3)* )?
        {
            type Output = $out3;
        }
    };
}

#[macro_export]
macro_rules! helper_list {
    (
        $(#[$meta:meta])*
        $vis:vis trait $name:ident < $($gen:ident),+ >;
        base $base_ty:ty $(where [ $($wbase:tt)* ])? => $base_out:ty;
        step < $head:ident, $tail:ident > $step_ty:ty $(where [ $($wstep:tt)* ])? => $step_out:ty;
    ) => {
        $(#[$meta])*
        $vis trait $name < $($gen),+ > {
            type Output;
        }

        impl< $($gen),+ > $name < $($gen),+ > for $base_ty
        $( where $($wbase)* )?
        {
            type Output = $base_out;
        }

        impl< $($gen),+ , $head, $tail > $name < $($gen),+ > for $step_ty
        $( where $($wstep)* )?
        {
            type Output = $step_out;
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
