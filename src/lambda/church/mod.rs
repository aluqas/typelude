//! Church Encodings
//!
//! Pure lambda calculus encodings of data structures:
//! - **Bool**: Church Booleans (`True`, `False`, `If`)
//! - **Numeral**: Church Numerals (`Zero`, `Succ`) and Arithmetic
//! - **Pair**: Church Pairs (`Pair`, `Fst`, `Snd`)

pub mod bool;
pub mod numeral;
pub mod pair;

// Re-export specific items to define the public API clearly

// Traits - Import from lambda::traits to re-export
pub use bool::{LFalse, LFalse1, LIf, LPureIf, LTrue, LTrue1};
pub use numeral::{
    LAdd, LAdd1, LAdd2, LAdd3, LExp, LMul, LMul1, LMul2, LMul3, LPred, LPredGen, LPredStep,
    LPureAdd, LPureExp, LPureMul, LPurePred, LPureSub, LSub, LSucc, LSucc1, LSuccGen, LZero,
    LZero1,
};
pub use pair::{LFst, LPair, LPureFst, LPureSnd, LSnd};

pub use crate::lambda::traits::{LBool, LNat, LTerm};
