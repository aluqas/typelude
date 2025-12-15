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
    // Standard Library
    std::{
        // Traits (Core)
        traits::EFunction,
        // Int (Arithmetic)
        int::{EAdd, ESub, EMul, EDiv, ERem, EPow, FAdd, FSub, FMul, FDiv, FRem, FPow},
        // Bool (Logical)
        bool::{
            TyTrue, TyFalse, AsBool, IsTrue, IsFalse, ToTyBool, ToTyBoolOut, Assert,
            ENot, EAnd, EOr, ENand, ENor, EXor, EXnor,
            FNot, FAnd, FOr, FNand, FNor, FXor, FXnor
        },
        // Array (Collection)
        array::{
            TyNil, TyArray, Cons,
            ELen, EHead, ETail, EIsEmpty, EGet, ESet, EConcat, EAppend, EPrepend, EContains,
            FLen, FHead, FTail, FIsEmpty, FGet, FSet, FConcat, FAppend, FPrepend, FContains
        },
        // Compare
        cmp::{
            EEq, ENotEq, ELt, ELe, EGt, EGe,
            FEq, FNeq, FLt, FLe, FGt, FGe
        }
    },
    // Macros
    tyarray,
};
