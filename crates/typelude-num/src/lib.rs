//! Type-level numeric utilities.

#![recursion_limit = "1024"]

pub use typenum::*;
extern crate typenum;

pub mod peano;

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
