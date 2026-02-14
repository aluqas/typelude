//! # Type-Level Strings
//!
//! Re-exports from the `tstr` crate to support type-level strings.
//!
//! Example:
//! ```rust
//! use typelude_std::std::prim::str::*;
//!
//! type Hello = TS!("Hello");
//! ```

pub use tstr::{TS, ts};

// Re-export kernel types
pub use crate::model::prim::str::Char;
pub use crate::std::prim::option::{None, Some};

// Convert Array of TyChar to &str?
// Requires const concatenation which is complex.
// We can use a recursive approach with a const block.

/*
// This would be ideal but requires advanced const generic expressions
impl<H, T> Reify<&'static str> for Array<H, T>
where
    H: Reify<char>,
    T: Reify<&'static str>
{
    const REIFIED: &'static str = ...;
}
*/

// For now, we rely on tstr for static strings, and Array<TyChar> for
// manipulation. We can provide conversion from Array<TyChar> to runtime string
// via Reify<[char; N]>.

// The Reify<u8> for Char is now handled in `crate::data::primitives::str::Char`

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::reify::Reify;
    #[cfg(feature = "nightly")]
    use crate::tyarray;

    #[test]
    fn test_char() {
        type A = Char<'a'>;
        assert_eq!(<A as Reify<char>>::REIFIED, 'a');
    }

    #[test]
    #[cfg(feature = "nightly")]
    fn test_array_of_char() {
        type S = tyarray![Char<'a'>, Char<'b'>, Char<'c'>];
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
