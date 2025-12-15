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
        // Array (Collection)
        array::{
            Cons, EAppend, EConcat, EContains, EGet, EHead, EIsEmpty, ELen, EPrepend, ESet, ETail,
            FAppend, FConcat, FContains, FGet, FHead, FIsEmpty, FLen, FPrepend, FSet, FTail,
            TyArray, TyNil,
        },
        // Bool (Logical)
        bool::{
            AsBool, Assert, EAnd, ENand, ENor, ENot, EOr, EXnor, EXor, FAnd, FNand, FNor, FNot,
            FOr, FXnor, FXor, IsFalse, IsTrue, ToTyBool, ToTyBoolOut, TyFalse, TyTrue,
        },
        // Compare
        cmp::{EEq, EGe, EGt, ELe, ELt, ENotEq, FEq, FGe, FGt, FLe, FLt, FNeq},
        // Int (Arithmetic)
        int::{EAdd, EDiv, EMul, EPow, ERem, ESub, FAdd, FDiv, FMul, FPow, FRem, FSub},
        // Ops markers (General)
        ops::EFunction,
    },
    // Macros
    tyarray,
};
