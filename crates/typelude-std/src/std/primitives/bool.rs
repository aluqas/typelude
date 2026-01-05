//! **Type-Level Boolean**
//!
//! Type-level booleans (`TyTrue`, `TyFalse`) and logical operations.

use typelude_core::Eval;
use typenum::{B0, B1};

// Re-export kernel types
// Re-export kernel types
pub use crate::data::primitives::bool::{Bool as TyBool, TyFalse, TyTrue};
use crate::std::traits::TypeBool;

//
// Adapter Implementation: TypeBool for TyTrue, TyFalse, B0, B1
//

// --- TyTrue / TyFalse ---

impl TypeBool for TyTrue {
    type Not = TyFalse;
    type And<Rhs: TypeBool> = Rhs;
    type Or<Rhs: TypeBool> = TyTrue;
    type Xor<Rhs: TypeBool> = Rhs::Not;
    type Nand<Rhs: TypeBool> = Rhs::Not;
    type Nor<Rhs: TypeBool> = TyFalse;
    type Xnor<Rhs: TypeBool> = Rhs;
}

impl TypeBool for TyFalse {
    type Not = TyTrue;
    type And<Rhs: TypeBool> = TyFalse;
    type Or<Rhs: TypeBool> = Rhs;
    type Xor<Rhs: TypeBool> = Rhs;
    type Nand<Rhs: TypeBool> = TyTrue;
    type Nor<Rhs: TypeBool> = Rhs::Not;
    type Xnor<Rhs: TypeBool> = Rhs::Not;
}

// --- B0 / B1 (typenum interoperability) ---

impl TypeBool for B0 {
    type Not = B1;
    type And<Rhs: TypeBool> = B0;
    type Or<Rhs: TypeBool> = Rhs;
    type Xor<Rhs: TypeBool> = Rhs;
    type Nand<Rhs: TypeBool> = B1;
    type Nor<Rhs: TypeBool> = Rhs::Not;
    type Xnor<Rhs: TypeBool> = Rhs::Not;
}

impl TypeBool for B1 {
    type Not = B0;
    type And<Rhs: TypeBool> = Rhs;
    type Or<Rhs: TypeBool> = B1;
    type Xor<Rhs: TypeBool> = Rhs::Not;
    type Nand<Rhs: TypeBool> = Rhs::Not;
    type Nor<Rhs: TypeBool> = B0;
    type Xnor<Rhs: TypeBool> = Rhs;
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

use crate::std::ops::TyFrom;

impl TyFrom<B1> for bool {
    type Output = TyTrue;
}
impl TyFrom<B0> for bool {
    type Output = TyFalse;
}

pub type ToTyBoolOut<T> = <bool as TyFrom<T>>::Output;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::{ELit, Evaluate};

    use super::*;
    use crate::std::ops::logic::*;

    #[test]
    fn test_not() {
        assert_type_eq_all!(Evaluate<ENot<ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENot<ELit<TyFalse>>>, TyTrue);
        // Interop test
        assert_type_eq_all!(Evaluate<ENot<ELit<B1>>>, B0);
    }
}
