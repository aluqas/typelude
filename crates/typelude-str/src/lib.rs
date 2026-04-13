//! 型レベル文字列プリミティブ。
//!
//! 文字列そのものの意味論を担保する軽量 carrier。
//! 高度な列演算は再実装せず、必要なら `IntoCol` で `typelude-col` へ
//! 明示変換して利用する。

#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![recursion_limit = "1024"]

extern crate self as typelude_str;

use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
pub use typelude_macros::tstr;
use typelude_std::core::{Apply, Op, Value};
use typenum::{B0, B1};

#[derive(Debug, Default)]
pub struct STail;

impl Value for STail {}

#[derive(Debug, Default)]
pub struct TStr<const CH: char, Tail>(PhantomData<Tail>);

impl<const CH: char, Tail> Value for TStr<CH, Tail> {}

#[derive(Debug, Default)]
pub struct SChar<const CH: char>;

impl<const CH: char> Value for SChar<CH> {}

pub trait IsStr {}

impl IsStr for STail {}

impl<const CH: char, Tail> IsStr for TStr<CH, Tail> where Tail: IsStr {}

pub trait BoolConst<const VALUE: bool> {
    type Output;
}

impl BoolConst<true> for () {
    type Output = B1;
}

impl BoolConst<false> for () {
    type Output = B0;
}

pub trait StrEq<Rhs> {
    type Output;
}

impl StrEq<STail> for STail {
    type Output = B1;
}

impl<const CH: char, Tail> StrEq<STail> for TStr<CH, Tail>
where
    Tail: IsStr,
{
    type Output = B0;
}

impl<const CH: char, Tail> StrEq<TStr<CH, Tail>> for STail
where
    Tail: IsStr,
{
    type Output = B0;
}

pub trait StrEqHelper<const MATCH: bool, RhsTail> {
    type Output;
}

impl<LhsTail, RhsTail> StrEqHelper<true, RhsTail> for LhsTail
where
    LhsTail: StrEq<RhsTail>,
{
    type Output = <LhsTail as StrEq<RhsTail>>::Output;
}

impl<LhsTail, RhsTail> StrEqHelper<false, RhsTail> for LhsTail {
    type Output = B0;
}

impl<const L: char, LhsTail, const R: char, RhsTail> StrEq<TStr<R, RhsTail>> for TStr<L, LhsTail>
where
    LhsTail: IsStr + StrEqHelper<{ L == R }, RhsTail>,
    RhsTail: IsStr,
{
    type Output = <LhsTail as StrEqHelper<{ L == R }, RhsTail>>::Output;
}

#[derive(Debug, Default)]
pub struct OpStrEq;

impl Value for OpStrEq {}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpStrEq
where
    Lhs: StrEq<Rhs>,
{
    type Output = <Lhs as StrEq<Rhs>>::Output;
}

pub type EStrEq<Lhs, Rhs> = Apply<OpStrEq, (Lhs, Rhs)>;

pub trait IntoCol {
    type Output;
}

impl IntoCol for STail {
    type Output = TTerm;
}

impl<const CH: char, Tail> IntoCol for TStr<CH, Tail>
where
    Tail: IntoCol + IsStr,
{
    type Output = TArr<SChar<CH>, <Tail as IntoCol>::Output>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_col::{TArr, TTerm};
    use typelude_std::core::{Evaluate, Op};
    use typenum::{B0, B1};

    use super::{EStrEq, IntoCol, OpStrEq, STail, StrEq, TStr};

    type ABC = TStr<'A', TStr<'B', TStr<'C', STail>>>;
    type ABD = TStr<'A', TStr<'B', TStr<'D', STail>>>;

    #[test]
    fn string_equality_distinguishes_shape_and_chars() {
        assert_type_eq_all!(<STail as StrEq<STail>>::Output, B1);
        assert_type_eq_all!(<ABC as StrEq<ABC>>::Output, B1);
        assert_type_eq_all!(<ABC as StrEq<ABD>>::Output, B0);
        assert_type_eq_all!(<ABC as StrEq<TStr<'A', STail>>>::Output, B0);
        assert_type_eq_all!(Evaluate<EStrEq<ABC, ABC>>, B1);
        assert_type_eq_all!(<OpStrEq as Op<(ABC, ABD)>>::Output, B0);
    }

    #[test]
    fn strings_convert_to_typelude_col_lists_explicitly() {
        assert_type_eq_all!(
            <ABC as IntoCol>::Output,
            TArr<super::SChar<'A'>, TArr<super::SChar<'B'>, TArr<super::SChar<'C'>, TTerm>>>
        );
    }
}
