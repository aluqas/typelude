//! Type-level boolean utilities.
//!
//! This crate owns boolean primitive values and boolean capability traits.
//! Shared higher-order APIs should use the canonical operator names from
//! `typelude_std::core`.
//!
//! Canonical operator mapping:
//! - `Not` -> `OpNot`
//! - `And` -> `OpAnd`
//! - `Or` -> `OpOr`
//! - `Xor` -> `OpXor`
//! - `Nand` -> `OpNand`

pub mod option;

pub use typelude_std::core::{And, Nand, Not, Or, Xor};

use typelude_std::core::{
    And as TlAnd, Nand as TlNand, Not as TlNot, Or as TlOr, Value, Xor as TlXor,
};

pub struct True;
pub struct False;

impl Value for True {}
impl Value for False {}

/// **Marker Trait**
/// Represents that a type is a boolean type (True or False).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a boolean type",
    label = "not a TYPE-LEVEL boolean",
    note = "[TYPELUDE MESSAGE] ensure `{Self}` is either `True` or `False`."
)]
pub trait IsBool {
    const VALUE: bool;

    type Not: IsBool;
    type Nand<B: IsBool>: IsBool;
    type And<B: IsBool>: IsBool;
    type Or<B: IsBool>: IsBool;
    type Xor<B: IsBool>: IsBool;
}

impl TlNot for True {
    type Output = <True as IsBool>::Not;
}

impl TlNot for False {
    type Output = <False as IsBool>::Not;
}

impl<Rhs: IsBool> TlAnd<Rhs> for True {
    type Output = <True as IsBool>::And<Rhs>;
}

impl<Rhs: IsBool> TlAnd<Rhs> for False {
    type Output = <False as IsBool>::And<Rhs>;
}

impl<Rhs: IsBool> TlOr<Rhs> for True {
    type Output = <True as IsBool>::Or<Rhs>;
}

impl<Rhs: IsBool> TlOr<Rhs> for False {
    type Output = <False as IsBool>::Or<Rhs>;
}

impl<Rhs: IsBool> TlXor<Rhs> for True {
    type Output = <True as IsBool>::Xor<Rhs>;
}

impl<Rhs: IsBool> TlXor<Rhs> for False {
    type Output = <False as IsBool>::Xor<Rhs>;
}

impl<Rhs: IsBool> TlNand<Rhs> for True
{
    type Output = <True as IsBool>::Nand<Rhs>;
}

impl<Rhs: IsBool> TlNand<Rhs> for False
{
    type Output = <False as IsBool>::Nand<Rhs>;
}

impl IsBool for True {
    const VALUE: bool = true;

    type Not = False;
    type Nand<B: IsBool> = B::Not;
    type And<B: IsBool> = B;
    type Or<B: IsBool> = True;
    type Xor<B: IsBool> = <B as IsBool>::Not;
}

impl IsBool for False {
    const VALUE: bool = false;

    type Not = True;
    type Nand<B: IsBool> = B::Not;
    type And<B: IsBool> = False;
    type Or<B: IsBool> = B;
    type Xor<B: IsBool> = B;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::{False, True};
    use typelude_std::core::{Apply, Evaluate, OpAnd, OpNand, OpNot, OpOr, OpXor};

    #[test]
    fn bool_primitives_work_through_canonical_ops() {
        assert_type_eq_all!(Evaluate<Apply<OpNot, True>>, False);
        assert_type_eq_all!(Evaluate<Apply<OpAnd, (True, False)>>, False);
        assert_type_eq_all!(Evaluate<Apply<OpOr, (False, True)>>, True);
        assert_type_eq_all!(Evaluate<Apply<OpXor, (True, False)>>, True);
        assert_type_eq_all!(Evaluate<Apply<OpNand, (True, True)>>, False);
    }
}
