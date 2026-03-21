pub mod col;
pub mod eval;
pub mod expr;
pub mod r#fn;

pub use col::{ECons, EList, ENil};
pub use eval::{Eval, Evaluate};
pub use expr::{EApp, ECall, ECall2, ECall3, ELit};
pub use r#fn::TyFn;
