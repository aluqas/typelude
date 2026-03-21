//! **Type-Level String Data**
//!
//! Canonical string data is represented as `Array<Char<_>, ...>`.

use crate::std::reify::Reify;

/// Type-level Character
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Char<const C: char>;

impl<const C: char> Reify<char> for Char<C> {
    const REIFIED: char = C;
}

impl<const C: char> Reify<u8> for Char<C> {
    const REIFIED: u8 = C as u8;
}

/// Macro for easily creating type-level character arrays.
#[macro_export]
macro_rules! tychars {
    () => { $crate::model::col::array::Nil };
    ($ch:literal $(,)?) => {
        $crate::model::col::array::Array<
            $crate::model::prim::str::Char<$ch>,
            $crate::model::col::array::Nil
        >
    };
    ($ch:literal, $($tail:literal),+ $(,)?) => {
        $crate::model::col::array::Array<
            $crate::model::prim::str::Char<$ch>,
            $crate::tychars![$($tail),+]
        >
    };
}

/// Alias of `tychars!` for string-like construction syntax.
#[macro_export]
macro_rules! tystr {
    ($($chars:literal),* $(,)?) => {
        $crate::tychars![$($chars),*]
    };
}
