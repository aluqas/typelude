use core::marker::PhantomData;

use tstr::IsTStr;
use typenum::{B0, B1, IsEqual};

pub trait NameEq<Rhs> {
    type Output;
}

impl<Lhs, Rhs> NameEq<Rhs> for Lhs
where
    Lhs: IsTStr + IsEqual<Rhs>,
    Rhs: IsTStr,
{
    type Output = <Lhs as IsEqual<Rhs>>::Output;
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
