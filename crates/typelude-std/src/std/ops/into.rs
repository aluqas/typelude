//! Conversion Traits (Type -> Type).
//!
//! - From: Type-level From

/// Type-level From.
///
/// Functions like `From<T>`, but for types.
/// Implemented on the **Target** type.
///
/// # Examples
///
/// ```rust
/// use typelude_std::std::{ops::From, prim::bool::{False, True}};
/// use typenum::{B0, B1};
///
/// // If impl From<B1> for bool { type Output = True; }
/// // type Res = <bool as From<B1>>::Output;
/// ```
///
/// ```rust,compile_fail
/// use typelude_std::std::ops::From;
/// use typelude_std::std::prim::bool::True;
/// use typenum::B1;
///
/// // No explicit conversion implementation exists.
/// fn main() {
///     let _: <True as From<B1>>::Output;
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "Cannot convert `{Source}` into `{Self}` via From",
    label = "conversion not implemented",
    note = "ensure `{Self}` implements `From<{Source}>`"
)]
pub trait From<Source> {
    type Output;
}
#[cfg(test)]
mod tests {
    use typenum::{B0, B1};

    use super::*;
    use crate::std::prim::bool::{False, True};

    #[test]
    fn test_from() {
        use static_assertions::assert_type_eq_all;
        assert_type_eq_all!(<bool as From<B1>>::Output, True);
        assert_type_eq_all!(<bool as From<B0>>::Output, False);
    }
}
