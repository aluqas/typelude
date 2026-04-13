use core::marker::PhantomData;

use typelude_str::{IsStr, StrEq};
use typenum::{B0, B1};

pub trait NameEq<Rhs> {
    type Output;
}

impl<Lhs, Rhs> NameEq<Rhs> for Lhs
where
    Lhs: IsStr + StrEq<Rhs>,
    Rhs: IsStr,
{
    type Output = <Lhs as StrEq<Rhs>>::Output;
}

pub trait BoolOutput {
    type Output;
}

impl BoolOutput for B0 {
    type Output = B0;
}

impl BoolOutput for B1 {
    type Output = B1;
}

#[derive(Debug, Default)]
pub struct NamePair<ModuleName, FieldName>(pub PhantomData<(ModuleName, FieldName)>);
