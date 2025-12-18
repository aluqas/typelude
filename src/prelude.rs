//! **Prelude**
//!
//! A module for convenient bulk imports.
//!
//! # Usage
//! ```ignore
//! use typelude::prelude::*;
//! ```

pub use crate::{
    // Evaluation infrastructure
    eval::{EApply, EApply2, EIf, ELit, EWhile, Eval, Evaluate},
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
            Assert, EAnd, ENand, ENor, ENot, EOr, EXnor, EXor, FAnd, FNand, FNor, FNot, FOr,
            FXnor, FXor, KindBool, ToTyBoolOut, TyFalse, TyTrue,
        },
        // Compare
        cmp::{EEq, EGe, EGt, ELe, ELt, ENotEq, FEq, FGe, FGt, FLe, FLt, FNeq},
        // Int (Arithmetic)
        int::{EAdd, EDiv, EMul, EPow, ERem, ESub, FAdd, FDiv, FMul, FPow, FRem, FSub},
        // Traits (Core)
        traits::TyFn,
    },
    // Macros
    tyarray,
};
