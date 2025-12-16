//! # Type-Level Strings
//!
//! Re-exports from the `tstr` crate to support type-level strings.
//!
//! Example:
//! ```rust
//! use typelude::{prelude::*, std::str::*};
//!
//! type Hello = TS!("Hello");
//! ```

pub use tstr::{TS, ts};

use crate::std::conv::ToConst;

/// Type-level Character
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TyChar<const C: char>;

impl<const C: char> ToConst<char> for TyChar<C> {
    const VALUE: char = C;
}

// Convert TyArray of TyChar to &str?
// Requires const concatenation which is complex.
// We can use a recursive approach with a const block.

/*
// This would be ideal but requires advanced const generic expressions
impl<H, T> ToConst<&'static str> for TyArray<H, T>
where
    H: ToConst<char>,
    T: ToConst<&'static str>
{
    const VALUE: &'static str = ...;
}
*/

// For now, we rely on tstr for static strings, and TyArray<TyChar> for manipulation.
// We can provide conversion from TyArray<TyChar> to runtime string via ToConst<[char; N]>.

impl<const C: char> ToConst<u8> for TyChar<C> {
    const VALUE: u8 = C as u8; // Only for ASCII?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tyarray;
    use crate::std::conv::ToConst;

    #[test]
    fn test_tychar() {
        type A = TyChar<'a'>;
        assert_eq!(<A as ToConst<char>>::VALUE, 'a');
    }

    #[test]
    fn test_tyarray_of_char() {
        type S = tyarray![TyChar<'a'>, TyChar<'b'>, TyChar<'c'>];
        let chars = <S as ToConst<[char; 3]>>::VALUE;
        assert_eq!(chars, ['a', 'b', 'c']);

        // Convert to string at runtime
        let s: String = chars.iter().collect();
        assert_eq!(s, "abc");
    }

    #[test]
    fn test_tstr_basic() {
        // Just ensure tstr macro works
        type _S = TS!("hello");
    }
}
