//! **Kernel Layer**
//!
//! Provides the core data structures and marker traits for type-level programming.
//! This layer contains NO evaluation logic, only "structure".

pub mod array;
pub mod bool;
pub mod traits;

pub use array::*;
pub use bool::*;
pub use traits::*;
