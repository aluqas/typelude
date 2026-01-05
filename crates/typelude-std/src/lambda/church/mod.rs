//! Church Encodings
//!
//! Pure lambda calculus encodings of data structures:
//! - **Bool**: Church Booleans (`True`, `False`, `If`)
//! - **Numeral**: Church Numerals (`Zero`, `Succ`) and Arithmetic
//! - **Pair**: Church Pairs (`Pair`, `Fst`, `Snd`)
//! - **Control**: Control flow combinators (`While`, `For`)

pub mod bool;
pub mod control;
pub mod numeral;
pub mod pair;

// Re-export specific items to define the public API clearly

// Traits - Import from lambda::traits to re-export
pub use bool::{LFalse, LFalse1, LIf, LIf1, LIf2, LPureIf, LTrue, LTrue1};
pub use control::{LFor, LWhile, LWhile1, LWhile2, LWhileHelper};
pub use numeral::{
    LAdd, LAdd1, LAdd2, LAdd3, LExp, LMul, LMul1, LMul2, LMul3, LPred, LPredStep, LSub, LSucc,
    LSucc1, LSuccGen, LZero, LZero1,
};
pub use pair::{LFst, LPair, LPair1, LPair2, LSnd};

pub use crate::lambda::traits::{LBool, LNat, LTerm};
