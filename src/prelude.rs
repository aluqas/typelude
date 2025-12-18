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
    eval::{EApp, EApp2, EIf, ELit, Eval, Evaluate},
    // Kernel Types & Traits
    kernel::{
        array::{Cons, TyArray, TyNil},
        bool::{TyBool, TyFalse, TyTrue},
        traits::Apply,
    },
    // Standard Library
    std::{
        // Array (Collection)
        array::{
            // Exprs
            EAppend,
            EConcat,
            EContains,
            EFilter,
            EFold,
            EGet,
            EHead,
            EIsEmpty,
            ELen,
            EMap,
            EPrepend,
            ESet,
            ETail,
            // Ops
            OpAppend,
            OpConcat,
            OpContains,
            OpFilter,
            OpFold,
            OpGet,
            OpHead,
            OpIsEmpty,
            OpLen,
            OpMap,
            OpPrepend,
            OpSet,
            OpTail,
        },
        // Bool (Logical)
        bool::{
            // Exprs
            Assert,
            EAnd,
            ENand,
            ENor,
            ENot,
            EOr,
            EXnor,
            EXor,
            // Traits
            KindBool,
            // Ops
            OpAnd,
            OpNand,
            OpNor,
            OpNot,
            OpOr,
            OpXnor,
            OpXor,
            ToTyBoolOut,
        },
        // Compare
        cmp::{
            // Exprs
            EEq,
            EGe,
            EGt,
            ELe,
            ELt,
            ENeq,
            // Ops
            OpEq,
            OpGe,
            OpGt,
            OpLe,
            OpLt,
            OpNeq,
        },
        // Int (Arithmetic)
        int::{
            // Exprs
            EAdd,
            EDiv,
            EMul,
            EPow,
            ERem,
            ESub,
            // Ops
            OpAdd,
            OpDiv,
            OpMul,
            OpPow,
            OpRem,
            OpSub,
        },
    },
    // Macros
    tyarray,
};
