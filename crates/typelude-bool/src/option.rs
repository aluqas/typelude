//! Type-level `Option` primitives owned by `typelude-bool`.

use core::marker::PhantomData;

use typelude_std::core::{Unwrap, UnwrapOr, Value};

use crate::{False, True};

/// Type-level `Some<T>`.
pub struct Some<T>(pub PhantomData<T>);

/// Type-level `None`.
pub struct None;

/// Capability trait for type-level option values.
pub trait IsOption {
    type IsSome;
    type IsNone;
}

impl<T> Value for Some<T> {}
impl Value for None {}

impl<T> IsOption for Some<T> {
    type IsSome = True;
    type IsNone = False;
}

impl IsOption for None {
    type IsSome = False;
    type IsNone = True;
}

impl<T> Unwrap for Some<T> {
    type Output = T;
}

impl<T, Default> UnwrapOr<Default> for Some<T> {
    type Output = T;
}

impl<Default> UnwrapOr<Default> for None {
    type Output = Default;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{Unwrap, UnwrapOr};

    use super::{IsOption, None, Some};
    use crate::{False, True};

    #[test]
    fn option_predicates_and_unwraps_work() {
        assert_type_eq_all!(<Some<u8> as IsOption>::IsSome, True);
        assert_type_eq_all!(<Some<u8> as IsOption>::IsNone, False);
        assert_type_eq_all!(<None as IsOption>::IsSome, False);
        assert_type_eq_all!(<None as IsOption>::IsNone, True);
        assert_type_eq_all!(<Some<u8> as Unwrap>::Output, u8);
        assert_type_eq_all!(<Some<u8> as UnwrapOr<u16>>::Output, u8);
        assert_type_eq_all!(<None as UnwrapOr<u16>>::Output, u16);
    }
}
