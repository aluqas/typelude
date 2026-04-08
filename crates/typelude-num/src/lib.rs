//! Type-level numeric utilities.
//!
//! This crate owns numeric primitive values and arithmetic capability traits.
//! Shared higher-order APIs should use the canonical operator names from
//! `typelude_std::core`.
//!
//! Canonical operator mapping:
//! - `Add` -> `OpAdd`
//! - `Sub` -> `OpSub`
//! - `Mul` -> `OpMul`
//! - `Div` -> `OpDiv`
//! - `Rem` -> `OpRem`
//! - `Pow` -> `OpPow`
//! - comparison families, when provided, map to `OpEq`, `OpNeq`, `OpLt`,
//!   `OpLe`, and `OpGt`
//!
//! `typenum` is re-exported for interoperability, but its foreign types are
//! not part of the shared self-based trait integration in this phase.

#![recursion_limit = "1024"]

pub use typelude_std::core::{Add, Div, Mul, Sub};
pub use typenum::*;
extern crate typenum;

pub mod peano;
