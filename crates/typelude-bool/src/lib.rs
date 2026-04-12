//! 型レベル論理値プリミティブと演算。
//!
//! ## 責務
//!
//! 論理値プリミティブ型（True、False）と能力トレイト（IsBool）を提供。
//! 型レベル高階APIは typelude_std::core のキャノニカル演算子を使用する。
//!
//! ## 名前空間マッピング
//!
//! - `Not` -> `OpNot`
//! - `And` -> `OpAnd`
//! - `Or` -> `OpOr`
//! - `Xor` -> `OpXor`
//! - `Nand` -> `OpNand`

pub mod option;

use typelude_std::core::{
    And as TlAnd, Nand as TlNand, Not as TlNot, Or as TlOr, Value, Xor as TlXor,
};
pub use typelude_std::core::{And, Nand, Not, Or, Xor};

/// 真。真値を表す型レベル値。
pub struct True;
/// 偽。偽値を表す型レベル値。
pub struct False;

impl Value for True {}
impl Value for False {}

/// 型レベル論理値能力トレイト。
///
/// 論理値（`True` または `False`）であることを示すマーカートレイト。
/// 本トレイトを実装していれば、当該を有効論理値とみなし、
/// 「Not, And, Or, Xor, Nand」による基本演算が使用できる。
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a boolean type",
    label = "not a TYPE-LEVEL boolean",
    note = "[TYPELUDE MESSAGE] ensure `{Self}` is either `True` or `False`."
)]
pub trait IsBool {
    /// 実行時の論理値（true または false）。
    const VALUE: bool;

    /// 論理的「不、否」を算出。
    type Not: IsBool;
    /// 論理的「NAND」演算を算出。
    type Nand<B: IsBool>: IsBool;
    /// 論理的「AND」演算を算出。
    type And<B: IsBool>: IsBool;
    /// 論理的「OR」演算を算出。
    type Or<B: IsBool>: IsBool;
    /// 論理的「XOR」演算を算出。
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

impl<Rhs: IsBool> TlNand<Rhs> for True {
    type Output = <True as IsBool>::Nand<Rhs>;
}

impl<Rhs: IsBool> TlNand<Rhs> for False {
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

mod primitive {
    use super::*;

    trait NandHelper<Lhs: IsBool, Rhs: IsBool> {
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

    type Nand<Lhs, Rhs> = <() as NandHelper<Lhs, Rhs>>::Output;
    type Not<T> = Nand<T, T>;
    type And<Lhs, Rhs> = Not<Nand<Lhs, Rhs>>;
    type Or<Lhs, Rhs> = Nand<Not<Lhs>, Not<Rhs>>;
    type Xor<Lhs, Rhs> = Nand<And<Lhs, Rhs>, Or<Lhs, Rhs>>;
}
#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{Apply, Evaluate, OpAnd, OpNand, OpNot, OpOr, OpXor};

    use super::{False, True};

    #[test]
    fn bool_primitives_work_through_canonical_ops() {
        assert_type_eq_all!(Evaluate<Apply<OpNot, True>>, False);
        assert_type_eq_all!(Evaluate<Apply<OpAnd, (True, False)>>, False);
        assert_type_eq_all!(Evaluate<Apply<OpOr, (False, True)>>, True);
        assert_type_eq_all!(Evaluate<Apply<OpXor, (True, False)>>, True);
        assert_type_eq_all!(Evaluate<Apply<OpNand, (True, True)>>, False);
    }
}
