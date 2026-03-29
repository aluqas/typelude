//! Type-level boolean utilities.

pub mod model {
    pub use typelude_std::model::prim::bool::*;
}

pub mod std {
    pub use typelude_std::std::prim::bool::*;
}

struct True;
struct False;

/// **Marker Trait**
/// Represents that a type is a boolean type (True or False).
///
/// And, Efficient implementations of boolean calculation operations
/// without trait resolution.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a boolean type",
    label = "not a TYPE-LEVEL boolean",
    note = "[TYPELUDE MESSAGE] ensure `{Self}` is either `True` or `False`."
)]
trait IsBool {
    const VALUE: bool;
    type Not: IsBool;
    type Nand<B: IsBool>: IsBool;
    type And<B: IsBool>: IsBool;
    type Or<B: IsBool>: IsBool;
    type Xor<B: IsBool>: IsBool;
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

/// **Nand Helper Trait**
/// Trait in type-level programming is conditional braunching and recursion.
/// so, we can implement Nand with only 4 impls, and then derive Not, And, Or,
/// Xor from it. This is not only more efficient, but more formally correct,
/// as it avoids the need for trait resolution in the implementation of Not,
/// And, Or, Xor.
trait NandHelper<Lhf: IsBool, Rhf: IsBool> {
    type Output: IsBool;
}

impl NandHelper<True, True> for () {
    type Output = False;
}
impl NandHelper<True, False> for () {
    type Output = True;
}
impl NandHelper<False, True> for () {
    type Output = True;
}
impl NandHelper<False, False> for () {
    type Output = True;
}

type Nand<Lhf, Rhf> = <() as NandHelper<Lhf, Rhf>>::Output;
type Not<A> = Nand<A, A>;
type And<Lhf, Rhf> = Not<Nand<Lhf, Rhf>>;
type Or<Lhf, Rhf> = Nand<Not<Lhf>, Not<Rhf>>;
type Xor<Lhf, Rhf> = Nand<Nand<Lhf, Not<Rhf>>, Nand<Not<Lhf>, Rhf>>;

trait IfHelper<Cond: IsBool, Then, Else> {
    type Output;
}
impl<Then, Else> IfHelper<True, Then, Else> for () {
    type Output = Then;
}
impl<Then, Else> IfHelper<False, Then, Else> for () {
    type Output = Else;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premitive_bool_operations() {
    }
}
