//! # Type-Level Strings
//!
//! Canonical strings are `Array<Char<_>, ...>`.

use typenum::Unsigned;

use crate::{
    model::col::array::{Array, IsList, Nil},
    std::{
        col::array::{Concat, Len},
        ops::EqDecide,
        prim::bool::{False, True},
        traits::Bool,
    },
};
pub use crate::{
    model::prim::str::Char,
    std::traits::{FromChars, ToChars},
};

/// Marker trait for canonical type-level strings.
pub trait IsCharArray: IsList {}

impl IsCharArray for Nil {}

impl<const C: char, Tail: IsCharArray> IsCharArray for Array<Char<C>, Tail> {}

impl ToChars for Nil {
    type Output = Nil;
}

impl<const C: char, Tail: IsCharArray> ToChars for Array<Char<C>, Tail> {
    type Output = Array<Char<C>, Tail>;
}

impl<T: IsCharArray> FromChars for T {
    type Output = T;
}

impl<const L: char, const R: char> EqDecide<Char<R>> for Char<L>
where
    (): crate::std::reify::ReflectBool<{ L == R }>,
{
    type Output = <() as crate::std::reify::ReflectBool<{ L == R }>>::Output;
}

/// UTF-8 byte length of a canonical string.
pub trait StrLen {
    type Output: Unsigned;
}

impl<S> StrLen for S
where
    S: IsCharArray + Len,
    <S as Len>::Output: Unsigned,
{
    type Output = <S as Len>::Output;
}

/// Equality between canonical strings.
pub trait StrEq<Rhs: IsCharArray> {
    type Output: Bool;
}

impl StrEq<Nil> for Nil {
    type Output = True;
}

impl<const C: char, Tail: IsCharArray> StrEq<Nil> for Array<Char<C>, Tail> {
    type Output = False;
}

impl<const C: char, Tail: IsCharArray> StrEq<Array<Char<C>, Tail>> for Nil {
    type Output = False;
}

impl<const LC: char, LTail: IsCharArray, const RC: char, RTail: IsCharArray>
    StrEq<Array<Char<RC>, RTail>> for Array<Char<LC>, LTail>
where
    Char<LC>: EqDecide<Char<RC>>,
    <Char<LC> as EqDecide<Char<RC>>>::Output: Bool,
    LTail: StrEq<RTail>,
{
    type Output =
        <<Char<LC> as EqDecide<Char<RC>>>::Output as Bool>::And<<LTail as StrEq<RTail>>::Output>;
}

/// Concatenation of canonical strings.
pub trait StrConcat<Rhs: IsCharArray> {
    type Output: IsCharArray;
}

impl<Lhs, Rhs> StrConcat<Rhs> for Lhs
where
    Lhs: IsCharArray + Concat<Rhs>,
    Rhs: IsCharArray,
    <Lhs as Concat<Rhs>>::Output: IsCharArray,
{
    type Output = <Lhs as Concat<Rhs>>::Output;
}

crate::typelude_macros::ty_fn! {
    /// UTF-8 byte length of a string.
    pub struct FStrLen<Str>
    where [Str: StrLen]
    {
        type Output = <Str as StrLen>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Equality of two strings.
    pub struct FStrEq<Lhs>
    {
        type Output = FStrEqCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Equality of two strings.
    pub struct FStrEqCaptured<Lhs, Rhs>
    where [Lhs: StrEq<Rhs>, Rhs: IsCharArray]
    {
        type Output = <Lhs as StrEq<Rhs>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Concatenate two strings.
    pub struct FStrConcat<Lhs>
    {
        type Output = FStrConcatCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Concatenate two strings.
    pub struct FStrConcatCaptured<Lhs, Rhs>
    where [Lhs: StrConcat<Rhs>, Rhs: IsCharArray]
    {
        type Output = <Lhs as StrConcat<Rhs>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Normalize a string into chars IR.
    pub struct FToChars<Str>
    where [Str: ToChars]
    {
        type Output = <Str as ToChars>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Normalize chars IR into the canonical string form.
    pub struct FFromChars<Chars>
    where [Chars: FromChars]
    {
        type Output = <Chars as FromChars>::Output;
    }
}

/// String length expression.
pub type EStrLen<Str> = crate::core::ECall<FStrLen, Str>;
/// String equality expression.
pub type EStrEq<Lhs, Rhs> = crate::core::ECall2<FStrEq, Lhs, Rhs>;
/// String concatenation expression.
pub type EStrConcat<Lhs, Rhs> = crate::core::ECall2<FStrConcat, Lhs, Rhs>;
/// Convert a string expression to chars.
pub type EToChars<Str> = crate::core::ECall<FToChars, Str>;
/// Convert chars-array expression to canonical string form.
pub type EFromChars<Chars> = crate::core::ECall<FFromChars, Chars>;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{ELit, Evaluate};

    use super::*;
    use crate::{
        std::{
            prim::bool::{False, True},
            reify::Reify,
        },
        tychars, tystr,
    };

    #[test]
    fn test_char() {
        type A = Char<'a'>;
        assert_eq!(<A as Reify<char>>::REIFIED, 'a');
    }

    #[test]
    fn test_macros() {
        type Hello1 = tystr!['h', 'e', 'l', 'l', 'o'];
        type Hello2 = tychars!['h', 'e', 'l', 'l', 'o'];
        assert_type_eq_all!(Hello1, Hello2);
    }

    #[test]
    fn test_to_chars_and_from_chars_are_identity() {
        type Hello = tystr!['h', 'e', 'l', 'l', 'o'];
        assert_type_eq_all!(<Hello as ToChars>::Output, Hello);
        assert_type_eq_all!(<Hello as FromChars>::Output, Hello);
        assert_type_eq_all!(Evaluate<EToChars<ELit<Hello>>>, Hello);
        assert_type_eq_all!(Evaluate<EFromChars<ELit<Hello>>>, Hello);
    }

    #[test]
    fn test_string_len_eq_concat() {
        type Hello = tystr!['h', 'e', 'l', 'l', 'o'];
        type World = tystr![' ', 'w', 'o', 'r', 'l', 'd'];
        type Joined = Evaluate<EStrConcat<ELit<Hello>, ELit<World>>>;

        assert_type_eq_all!(Evaluate<EStrLen<ELit<Hello>>>, typenum::U5);
        assert_type_eq_all!(Evaluate<EStrEq<ELit<Hello>, ELit<Hello>>>, True);
        assert_type_eq_all!(
            Evaluate<EStrEq<ELit<Hello>, ELit<tystr!['h', 'e', 'l', 'l', 'a']>>>,
            False
        );
        assert_type_eq_all!(Joined, tystr!['h', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd']);
    }

    #[test]
    #[cfg(feature = "nightly")]
    fn test_array_of_char() {
        type S = tystr!['a', 'b', 'c'];
        let chars = <S as Reify<[char; 3]>>::REIFIED;
        assert_eq!(chars, ['a', 'b', 'c']);

        let s: String = chars.iter().collect();
        assert_eq!(s, "abc");
    }
}
