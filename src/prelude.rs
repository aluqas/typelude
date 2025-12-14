//! **Prelude**
//!
//! 便利な一括インポート用モジュール。
//!
//! # Usage
//! ```ignore
//! use typelude::prelude::*;
//! ```

pub use crate::{
    // Evaluation infrastructure
    eval::{EApply, EApply2, EIf, ELit, EWhile, Evaluable, Evaluator},
    // Function trait and markers
    func::{
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
        // Array functions
        EGet,
        EHead,
        EIsEmpty,
        ELen,
        // Boolean functions
        ENand,
        ENor,
        ENot,
        ENotEq,
        EOr,
        EPrepend,
        ETail,
        // Logical functions
        EXnor,
        EXor,
        // Function markers
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
    },
    types::{
        array::{Cons, TyArray, TyNil},
        bool::{AsBool, IsFalse, IsTrue, ToTyBool, ToTyBoolOut, TyFalse, TyTrue},
        eq::{_TypeEqConst, AssertBool},
    },
};
