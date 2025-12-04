//! **Prelude**
//!
//! 便利な一括インポート用モジュール。
//!
//! # Usage
//! ```ignore
//! use typelude::prelude::*;
//! ```

// Evaluation infrastructure
pub use crate::eval::{EApply, EApply2, EIf, ELit, EWhile, Evaluable, Evaluator};

// Function trait and markers
pub use crate::func::{
    // Boolean functions
    EAnd,
    // Array functions
    EAppend,
    EConcat,
    EContains,
    // Equality functions
    EEq,
    // Function trait
    EFunction,
    EGet,
    EHead,
    EIsEmpty,
    ELen,
    ENand,
    ENor,
    ENot,
    ENotEq,
    EOr,
    EPrepend,
    ETail,
    EXnor,
    EXor,
    FAnd,
    FAppend,
    FConcat,
    FContains,
    FEq,
    FGet,
    FHead,
    FIsEmpty,
    FLen,
    FNand,
    FNor,
    FNot,
    FNotEq,
    FOr,
    FPrepend,
    FTail,
    FXnor,
    FXor,
};

// Type-level boolean
pub use crate::types::bool::{AsBool, IsFalse, IsTrue, ToTyBool, ToTyBoolOut, TyFalse, TyTrue};

// Type-level array
pub use crate::types::array::{Cons, TyArray, TyNil};

// Equality helpers (internal, but useful)
pub use crate::types::eq::{_TypeEqConst, AssertBool};

// Re-export tyarray macro
pub use crate::tyarray;
