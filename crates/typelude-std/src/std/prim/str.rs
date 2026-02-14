//! # Type-Level Strings
//!
//! Re-exports from `tstr` and core string primitives.

pub use tstr::{TS, ts};

// Re-export kernel types
pub use crate::model::prim::str::Char;
pub use crate::std::prim::option::{None, Some};

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
