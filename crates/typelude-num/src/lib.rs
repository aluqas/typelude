//! Type-level numeric utilities.

#![recursion_limit = "1024"]

pub mod model {
    pub use typelude_std::model::prim::int::*;
}

pub mod std {
    pub use typelude_std::std::prim::int::*;
}

pub use typenum::*;
extern crate typenum;

mod peano;

/// Ops
trait Add<Lhs, Rhs> {
    type Output;
}
trait Sub<Lhs, Rhs> {
    type Output;
}
trait Mul<Lhs, Rhs> {
    type Output;
}
trait Div<Lhs, Rhs> {
    type Output;
}
