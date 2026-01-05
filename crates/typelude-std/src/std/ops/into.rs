//! Conversion Traits (Type -> Type).
//!
//! - From: Type-level From
//! - Into: Type-level Into

/// Type-level From.
///
/// Functions like `From<T>`, but for types.
/// Implemented on the **Target** type.
///
/// # Examples
///
/// ```rust
/// use typelude_std::std::{
///     ops::{From, Into},
///     prim::bool::{False, True},
/// };
/// use typenum::{B0, B1};
///
/// // If impl From<B1> for bool { type Output = True; }
/// // type Res = <bool as From<B1>>::Output;
/// ```
#[diagnostic::on_unimplemented(
    message = "Cannot convert `{Source}` into `{Self}` via From",
    label = "conversion not implemented",
    note = "ensure `{Self}` implements `From<{Source}>`"
)]
pub trait From<Source> {
    type Output;
}

/// Type-level Into.
///
/// Blanket implemented for any type that implements [`From`] on the target.
#[diagnostic::on_unimplemented(
    message = "Cannot convert `{Self}` into `{Target}` via Into",
    label = "conversion not implemented",
    note = "ensure `{Target}` implements `From<{Self}>`"
)]
pub trait Into<Target> {
    type Output;
}

impl<T, Target> Into<Target> for T
where
    Target: From<T>,
{
    type Output = <Target as From<T>>::Output;
}
#[cfg(test)]
mod tests {
    use typenum::{B0, B1};

    use super::*;
    use crate::std::prim::bool::{False, True};

    // Implementations for bool are now in std::bool.rs
    // checking they work via Into

    #[test]
    fn test_into() {
        use static_assertions::assert_type_eq_all;
        assert_type_eq_all!(<B1 as Into<bool>>::Output, True);
        assert_type_eq_all!(<B0 as Into<bool>>::Output, False);
    }
}
