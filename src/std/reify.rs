//! Reify

/*
// This trait can be compiled because rustc's const generic constraints.
pub trait FromValue<T, const VALUE: T> {
    type Output;
}
*/

/// KindType to runtime type.
///
/// Example
/// - TyTrue, TyFalse -> bool
/// - U1, U2, U3 -> usize
/// - tyarray![U1, U2, U3] -> [usize; 3]
/// - tyarray![TyTrue, U1] -> (bool, usize)
pub trait ToType<T> {
    type Output;
}

/// Ty to runtime value.
///
/// This trait allows converting type-level constants (like `TyTrue`, `U1`)
/// into their runtime equivalents (`true`, `1`).
///
/// # Examples
///
/// ```rust
/// use typenum::{U1, U2};
/// use typeutils::std::{
///     bool::{TyFalse, TyTrue},
///     reify::ToValue,
/// };
///
/// assert_eq!(<TyTrue as ToValue<bool>>::VALUE, true);
/// assert_eq!(<TyFalse as ToValue<bool>>::VALUE, false);
/// assert_eq!(<U1 as ToValue<usize>>::VALUE, 1);
/// ```
pub trait ToValue<T> {
    const VALUE: T;

    fn value() -> T {
        Self::VALUE
    }
}

//
// Identifier: Boolean
//

use crate::std::bool::{TyFalse, TyTrue};

impl ToValue<bool> for TyTrue {
    const VALUE: bool = true;
}

impl ToValue<bool> for TyFalse {
    const VALUE: bool = false;
}

//
// Identifier: Integer (typenum)
//

use typenum::{Integer, Unsigned};

impl<U: Unsigned> ToValue<usize> for U {
    const VALUE: usize = U::USIZE;
}

impl<U: Unsigned> ToValue<u64> for U {
    const VALUE: u64 = U::U64;
}

impl<U: Unsigned> ToValue<u32> for U {
    const VALUE: u32 = U::U32;
}

impl<U: Unsigned> ToValue<u16> for U {
    const VALUE: u16 = U::U16;
}

impl<U: Unsigned> ToValue<u8> for U {
    const VALUE: u8 = U::U8;
}

impl<I: Integer> ToValue<isize> for I {
    const VALUE: isize = I::ISIZE;
}

impl<I: Integer> ToValue<i64> for I {
    const VALUE: i64 = I::I64;
}

impl<I: Integer> ToValue<i32> for I {
    const VALUE: i32 = I::I32;
}

impl<I: Integer> ToValue<i16> for I {
    const VALUE: i16 = I::I16;
}

impl<I: Integer> ToValue<i8> for I {
    const VALUE: i8 = I::I8;
}

use crate::std::array::*;

#[cfg(test)]
mod tests {
    use typenum::{N1, U1};

    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(<TyTrue as ToValue<bool>>::VALUE, true);
        assert_eq!(<TyFalse as ToValue<bool>>::VALUE, false);
    }

    #[test]
    fn test_int() {
        assert_eq!(<U1 as ToValue<usize>>::VALUE, 1);
        assert_eq!(<U1 as ToValue<u8>>::VALUE, 1);
        assert_eq!(<N1 as ToValue<isize>>::VALUE, -1);
    }
}
