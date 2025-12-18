//! Pure Lambda Calculus Module
//!
//! This module implements theoretical foundations of computation using pure type-level programming.
//! It does not rely on `std` or `typenum` logic, building everything from axioms.
//!
//! ## Structure
//!
//! - **traits**: Core traits (`Lambda`, `Bind`)
//! - **church**: Church encodings (Booleans, Numerals, Pairs)
//! - **monads**: Monad implementations (Identity, State, Either, CPS)
//! - **ski**: SKI combinators
//! - **fix**: Y-combinator (fixed-point)
//! - **curry**: Curry/Uncurry transformations
//! - **list**: Scott-encoded lists
//! - **proof**: Type-level theorem proving

pub mod church;
pub mod curry;
pub mod fix;
pub mod list;
pub mod monads;
pub mod proof;
pub mod ski;
pub mod traits;

// Re-export core traits
pub use traits::{Bind, Lambda};

// Re-export Apply from kernel for convenience
pub use crate::kernel::traits::Apply;

// Backwards compatibility: re-export all items from subdirectories
pub use church::*;
pub use monads::*;
