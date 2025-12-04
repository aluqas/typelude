mod private {
    pub(crate) trait Sealed {}
}

trait IsBit: private::Sealed {}

/// ビットを表す構造体: 0
struct B0;
impl private::Sealed for B0 {}
impl IsBit for B0 {}

trait IsZero: IsBit {}
impl IsZero for B0 {}

/// ビットを表す構造体: 1
struct B1;
impl private::Sealed for B1 {}
impl IsBit for B1 {}
trait IsOne: IsBit {}
impl IsOne for B1 {}

// -----------------------------------------------------------------------------

struct LogicTarget;

impl private::Sealed for LogicTarget {}

trait Nand<A: IsBit, B: IsBit> { type Output: IsBit; }
impl Nand<B0, B0> for LogicTarget { type Output = B1; }
impl Nand<B1, B0> for LogicTarget { type Output = B1; }
impl Nand<B0, B1> for LogicTarget { type Output = B1; }
impl Nand<B1, B1> for LogicTarget { type Output = B0; }

type NandOut<A, B> = <LogicTarget as Nand<A, B>>::Output;

trait Not<A: IsBit> { type Output: IsBit; }
impl<A: IsBit> Not<A> for LogicTarget
where LogicTarget: Nand<A, A>{
    type Output = <LogicTarget as Nand<A, A>>::Output;
}
type NotOut<A> = <LogicTarget as Not<A>>::Output;

trait And<A: IsBit, B: IsBit> { type Output: IsBit; }
impl<A: IsBit, B: IsBit> And<A, B> for LogicTarget
where LogicTarget: Nand<A, B> + Not<NandOut<A, B>> {
    type Output = <LogicTarget as Not<NandOut<A, B>>>::Output;
}
type OrOut<A, B> = <LogicTarget as And<A, B>>::Output;

trait Or<A: IsBit, B: IsBit> { type Output: IsBit; }
impl<A: IsBit, B: IsBit> Or<A, B> for LogicTarget
where LogicTarget: Not<A> + Not<B> + Nand<NotOut<A>, NotOut<B>> {
    type Output = <LogicTarget as Nand<NotOut<A>, NotOut<B>>>::Output;
}

trait Xor<A: IsBit, B: IsBit> { type Output: IsBit; }
impl<A: IsBit, B: IsBit> Xor<A, B> for LogicTarget
where LogicTarget: Or<A, B> + Nand<A, B> + And<OrOut<A, B>, NandOut<A, B>> {
    type Output = <LogicTarget as And<OrOut<A, B>, NandOut<A, B>>>::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logic() {
        assert_eq!(<LogicTarget as Nand<B0, B0>>::Output, B1);
        assert_eq!(<LogicTarget as Nand<B1, B0>>::Output, B1);
        assert_eq!(<LogicTarget as Nand<B0, B1>>::Output, B1);
        assert_eq!(<LogicTarget as Nand<B1, B1>>::Output, B0);
    }
}