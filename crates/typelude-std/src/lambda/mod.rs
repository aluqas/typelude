// Modules
pub mod church;
pub mod curry;
pub mod fix;
pub mod list;
pub mod monads;
pub mod proof;
pub mod ski;
pub mod thunk;
pub mod traits;

// =========================================================================
// Facade Exports
// =========================================================================

// 1. Core Traits
// 2. Church Encodings (Terms)
pub use church::*;
// 3. Lists
pub use list::{LCons, LFoldr, LHeadOr, LIsEmpty, LNil, LTailOr};
// 4. Monads
pub use monads::*;
// 5. Control
pub use thunk::{LForce, LThunk};
pub use traits::{LApp, LBind, LBool, LList, LNat, LTerm, Lambda};

// 6. Helpers
