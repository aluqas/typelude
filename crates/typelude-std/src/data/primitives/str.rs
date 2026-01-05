//! **Type-Level String Data**
//!
//! Re-exports from the `tstr` crate to support type-level strings.

pub use tstr::{TS, ts};

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
