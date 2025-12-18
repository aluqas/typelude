//! Pure Lambda Calculus Module
//!
//! This module implements theoretical foundations of computation using pure type-level programming.
//! It does not rely on `std` or `typenum` logic, building everything from axioms.

pub mod church;
pub mod either;
pub mod fib;
pub mod fix;
pub mod identity;
pub mod list;
pub mod monad;
pub mod ski;
pub mod state;
pub mod traits;

pub use crate::kernel::traits::Apply;
pub use traits::Lambda;
