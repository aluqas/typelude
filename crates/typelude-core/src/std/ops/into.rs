//! Conversion Traits (Type -> Type).
//!
//! - TyFrom: Type-level From
//! - TyInto: Type-level Into

/// Type-level From.
///
/// Functions like `From<T>`, but for types.
/// Implemented on the **Target** type.
///
/// # Examples
///
/// ```rust
/// use typelude_core::std::{
///     bool::{TyFalse, TyTrue},
///     ops::{TyFrom, TyInto},
/// };
/// use typenum::{B0, B1};
///
/// // If impl TyFrom<B1> for bool { type Output = TyTrue; }
/// // type Res = <bool as TyFrom<B1>>::Output;
/// ```
pub trait TyFrom<Source> {
    type Output;
}

/// Type-level Into.
///
/// Blanket implemented for any type that implements [`TyFrom`] on the target.
pub trait TyInto<Target> {
    type Output;
}

impl<T, Target> TyInto<Target> for T
where
    Target: TyFrom<T>,
{
    type Output = <Target as TyFrom<T>>::Output;
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use typenum::{B0, B1};

    use super::*;
    use crate::std::bool::{TyFalse, TyTrue};

    // Implementations for bool are now in std::bool.rs
    // checking they work via TyInto

    #[test]
    fn test_ty_into() {
        use static_assertions::assert_type_eq_all;
        assert_type_eq_all!(<B1 as TyInto<bool>>::Output, TyTrue);
        assert_type_eq_all!(<B0 as TyInto<bool>>::Output, TyFalse);
    }
}
