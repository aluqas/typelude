use typelude_std::core::{Get, Set};

pub trait LocalGet<Idx> {
    type Output;
}

impl<Locals, Idx> LocalGet<Idx> for Locals
where
    Locals: Get<Idx>,
{
    type Output = <Locals as Get<Idx>>::Output;
}

pub trait LocalSet<Idx, Value> {
    type Output;
}

impl<Locals, Idx, Value> LocalSet<Idx, Value> for Locals
where
    Locals: Set<Idx, Value>,
{
    type Output = <Locals as Set<Idx, Value>>::Output;
}
