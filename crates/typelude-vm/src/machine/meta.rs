use core::marker::PhantomData;

use typelude_std::{core::Eval, std::col::array::Nil};

#[derive(Debug)]
pub struct VmMeta<TraceLog, Fuel, World>(pub PhantomData<(TraceLog, Fuel, World)>);

impl<T, F, W> Eval for VmMeta<T, F, W> {
    type Output = Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoFuel;

impl Eval for NoFuel {
    type Output = Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoWorld;

impl Eval for NoWorld {
    type Output = Self;
}

pub type DefaultMeta = VmMeta<Nil, NoFuel, NoWorld>;
