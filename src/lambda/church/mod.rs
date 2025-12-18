//! Church Encodings
//!
//! Pure lambda calculus encodings of data structures:
//! - **Bool**: Church Booleans (`True`, `False`, `If`)
//! - **Numeral**: Church Numerals (`Zero`, `Succ`) and Arithmetic
//! - **Pair**: Church Pairs (`Pair`, `Fst`, `Snd`)

pub mod bool;
pub mod numeral;
pub mod pair;

// Re-export all items for backwards compatibility
pub use bool::*;
pub use numeral::*;
pub use pair::*;
