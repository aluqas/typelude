//! Monad Implementations
//!
//! Type-level monads for various computational effects:
//! - **Identity**: `Id<T>` - trivial monad
//! - **State**: `State<F>` - stateful computations
//! - **Either**: `Left<L>` / `Right<R>` - error handling
//! - **CPS**: `Cont<F>` - continuation-passing style

pub mod cps;
pub mod either;
pub mod identity;
pub mod state;

// Re-export all items for backwards compatibility
// Re-export specific items
pub use cps::LCont;
pub use either::{LLeft, LRight};
pub use identity::LId;
pub use state::{LBindGet, LBindPut, LBindState, LGet, LPut, LReturn, LState};
