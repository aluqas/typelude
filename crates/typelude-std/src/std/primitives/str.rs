//! # Type-Level Strings
//!
//! Re-exports from the `tstr` crate to support type-level strings.
//!
//! Example:
//! ```rust
//! use typelude_std::std::str::*;
//!
//! type Hello = TS!("Hello");
//! ```

pub use tstr::{TS, ts};

use crate::std::reify::Reify;

/// Type-level Character
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TyChar<const C: char>;

impl<const C: char> Reify<char> for TyChar<C> {
    const REIFIED: char = C;
}

// Convert TyArray of TyChar to &str?
// Requires const concatenation which is complex.
// We can use a recursive approach with a const block.

/*
// This would be ideal but requires advanced const generic expressions
impl<H, T> Reify<&'static str> for TyArray<H, T>
where
    H: Reify<char>,
    T: Reify<&'static str>
{
    const REIFIED: &'static str = ...;
}
*/

// For now, we rely on tstr for static strings, and TyArray<TyChar> for manipulation.
// We can provide conversion from TyArray<TyChar> to runtime string via Reify<[char; N]>.

impl<const C: char> Reify<u8> for TyChar<C> {
    const REIFIED: u8 = C as u8; // Only for ASCII?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::reify::Reify;
    #[cfg(feature = "nightly")]
    use crate::tyarray;

    #[test]
    fn test_tychar() {
        type A = TyChar<'a'>;
        assert_eq!(<A as Reify<char>>::REIFIED, 'a');
    }

    #[test]
    #[cfg(feature = "nightly")]
    fn test_tyarray_of_char() {
        type S = tyarray![TyChar<'a'>, TyChar<'b'>, TyChar<'c'>];
        // Reify to array [char; 3]
        let chars = <S as Reify<[char; 3]>>::REIFIED;
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
