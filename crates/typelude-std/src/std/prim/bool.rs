//! **Type-Level Boolean**
//!
//! Type-level booleans (`True`, `False`) and logical operations.

use typelude_core::Eval;
use typenum::{B0, B1};

// Re-export kernel types
pub use crate::data::primitives::bool::{False, IsBool, True};
pub use crate::std::traits::Bool;

//
// Adapter Implementation: Bool for True, False, B0, B1
//

// --- True / False ---

impl Bool for True {
    type Not = False;
    type And<Rhs: Bool> = Rhs;
    type Or<Rhs: Bool> = True;
    type Xor<Rhs: Bool> = Rhs::Not;
    type Nand<Rhs: Bool> = Rhs::Not;
    type Nor<Rhs: Bool> = False;
    type Xnor<Rhs: Bool> = Rhs;
}

impl Bool for False {
    type Not = True;
    type And<Rhs: Bool> = False;
    type Or<Rhs: Bool> = Rhs;
    type Xor<Rhs: Bool> = Rhs;
    type Nand<Rhs: Bool> = True;
    type Nor<Rhs: Bool> = Rhs::Not;
    type Xnor<Rhs: Bool> = Rhs::Not;
}

// --- B0 / B1 (typenum interoperability) ---

impl Bool for B0 {
    type Not = B1;
    type And<Rhs: Bool> = B0;
    type Or<Rhs: Bool> = Rhs;
    type Xor<Rhs: Bool> = Rhs;
    type Nand<Rhs: Bool> = B1;
    type Nor<Rhs: Bool> = Rhs::Not;
    type Xnor<Rhs: Bool> = Rhs::Not;
}

impl Bool for B1 {
    type Not = B0;
    type And<Rhs: Bool> = Rhs;
    type Or<Rhs: Bool> = B1;
    type Xor<Rhs: Bool> = Rhs::Not;
    type Nand<Rhs: Bool> = Rhs::Not;
    type Nor<Rhs: Bool> = B0;
    type Xnor<Rhs: Bool> = Rhs;
}

//
// Bool Conversion Utilities
//

pub struct Assert<const COND: bool>;

impl<const COND: bool> Eval for Assert<COND>
where
    (): crate::std::reify::ReflectBool<COND>,
    <() as crate::std::reify::ReflectBool<COND>>::Output: Eval,
{
    type Output = <() as crate::std::reify::ReflectBool<COND>>::Output;
}

use crate::std::ops::From;

impl From<B1> for bool {
    type Output = True;
}
impl From<B0> for bool {
    type Output = False;
}

pub type ToBoolOut<T> = <bool as From<T>>::Output;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::{ELit, Evaluate};

    use super::*;
    use crate::std::ops::logic::*;

    #[test]
    fn test_not() {
        assert_type_eq_all!(Evaluate<ENot<ELit<True>>>, False);
        assert_type_eq_all!(Evaluate<ENot<ELit<False>>>, True);
        // Interop test
        assert_type_eq_all!(Evaluate<ENot<ELit<B1>>>, B0);
    }
}
