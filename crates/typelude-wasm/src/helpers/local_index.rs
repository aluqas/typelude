use typelude_std::core::{Get, Set};

/// local index から値を取得する helper。
pub trait LocalGet<Idx> {
    type Output;
}

impl<Locals, Idx> LocalGet<Idx> for Locals
where
    Locals: Get<Idx>,
{
    type Output = <Locals as Get<Idx>>::Output;
}

/// local index に値を書き込む helper。
pub trait LocalSet<Idx, Value> {
    type Output;
}

impl<Locals, Idx, Value> LocalSet<Idx, Value> for Locals
where
    Locals: Set<Idx, Value>,
{
    type Output = <Locals as Set<Idx, Value>>::Output;
}
