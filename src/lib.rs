//! # Typelude
//!
//! A library for type-level programming in Rust.
//!
//! ## Features
//!
//! - Unified evaluation interface via `Evaluable` trait
//! - Function and expression separation using `EApply<F, A>` pattern
//! - Type-level booleans and logical operations
//! - Type-level arrays and array operations
//!
//! ## Quick Start
//!
//! ```ignore
//! use typelude::prelude::*;
//!
//! // Conditional Branching
//! type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
//! // Result = i32
//!
//! // Array Operations
//! type Len = Evaluator<ELen<ELit<tyarray![i32, f64, bool]>>>;
//! // Len = U3
//! ```
//!
//! ## Architecture
//!
//! ```text
//! eval/       - Evaluation Infrastructure (Evaluable, ELit, EIf, EWhile, EApply)
//! std/        - Standard Library (int, bool, array, cmp, ops)
//! machine/    - Stack Machine Implementation
//! prelude     - Convenient bulk imports
//! ```

// Unstable features required for type-level programming
#![feature(specialization)]
#![feature(generic_const_exprs)]
#![feature(inherent_associated_types)]
#![allow(incomplete_features)]

// Export typenum for macros
pub use typenum;

pub mod eval;
pub mod kernel;
pub mod machine;
pub mod macros;
pub mod prelude;
pub mod std;
