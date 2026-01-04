pub mod app;
pub mod bridge;
pub mod traits;
pub mod impls; // Typenum support

pub use app::*;
pub use bridge::*;
pub use traits::*;

/// Sealed trait pattern
#[doc(hidden)]
pub trait Sealed {}

/// Trait to evaluate type-level expressions
pub trait Eval {
    type Output;
}

/// Type alias to obtain evaluation results of expressions
pub type Evaluate<T> = <T as Eval>::Output;
