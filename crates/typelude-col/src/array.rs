pub struct TArr<Val, A> {
    first: Val,
    rest: A,
}
pub struct TTerm;

#[macro_export]
macro_rules! tarr {
    () => ( $crate::ATerm );
    ($n:ty) => ( $crate::TArr<$n, $crate::ATerm> );
    ($n:ty,) => ( $crate::TArr<$n, $crate::ATerm> );
    ($n:ty, $($tail:ty),+) => ( $crate::TArr<$n, tarr![$($tail),+]> );
    ($n:ty, $($tail:ty),+,) => ( $crate::TArr<$n, tarr![$($tail),+]> );
    ($n:ty | $rest:ty) => ( $crate::TArr<$n, $rest> );
    ($n:ty, $($tail:ty),+ | $rest:ty) => ( $crate::TArr<$n, tarr![$($tail),+ | $rest]> );
}

/// **Marker Trait**
/// Represents that a type is a type-level array.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a type-level array",
    label = "not a TYPE-LEVEL array",
    note = "ensure `{Self}` is of the form `TArr<_, _>`"
)]
trait TypeArray {
    const LEN: usize;
}

impl TypeArray for TTerm {
    // 0-indexed
    const LEN: usize = 0;
}

impl<Val, A: TypeArray> TypeArray for TArr<Val, A> {
    const LEN: usize = A::LEN + 1;
}

const fn tarr_len<T: TypeArray>() -> usize {
    T::LEN
}

use typelude_num::peano::{Nat, Succ, Zero};

/// Index access
#[diagnostic::on_unimplemented(
    message = "Cannot access index `{Idx}` in `{Self}`",
    label = "index access failed",
    note = "index might be out of bounds or `{Self}` is not a list"
)]
trait Get<Idx: Nat> {
    type Output;
}

impl<Head, Tail: TypeArray> Get<Zero> for TArr<Head, Tail> {
    type Output = Head;
}

impl<Head, Tail: TypeArray, Idx: Nat> Get<Succ<Idx>> for TArr<Head, Tail>
where
    Tail: Get<Idx>,
{
    type Output = <Tail as Get<Idx>>::Output;
}

/// Index Set
trait Set<Idx: Nat, Val> {
    type Output;
}

impl<Head, Tail: TypeArray, Val> Set<Zero, Val> for TArr<Head, Tail> {
    type Output = TArr<Val, Tail>;
}

impl<Head, Tail: TypeArray, Val, Idx: Nat> Set<Succ<Idx>, Val> for TArr<Head, Tail>
where
    Tail: Set<Idx, Val>,
{
    type Output = TArr<Head, <Tail as Set<Idx, Val>>::Output>;
}

/// Concat
trait Concat<Other: TypeArray> {
    type Output: TypeArray;
}

impl<Other: TypeArray> Concat<Other> for TTerm {
    type Output = Other;
}

impl<Head, Tail: TypeArray, Other: TypeArray> Concat<Other> for TArr<Head, Tail>
where
    Tail: Concat<Other>,
{
    type Output = TArr<Head, <Tail as Concat<Other>>::Output>;
}

/// Append
trait Append<Other> {
    type Output: TypeArray;
}

impl<Other> Append<Other> for TTerm {
    type Output = TArr<Other, TTerm>;
}

impl<Head, Tail: TypeArray, Other> Append<Other> for TArr<Head, Tail>
where
    Tail: Append<Other>,
{
    type Output = TArr<Head, <Tail as Append<Other>>::Output>;
}

/// Prepend
trait Prepend<Other> {
    type Output: TypeArray;
}

impl<Other, Array: TypeArray> Prepend<Other> for Array {
    type Output = TArr<Other, Array>;
}

/*
/// Contains
/// Nightly only or incomplete-inequality feature (macro)
trait Contains<T> {
    const VALUE: bool;
}

impl<T> Contains<T> for TTerm {
    const VALUE: bool = false;
}

impl<Tail: TypeArray, T> Contains<T> for TArr<T, Tail>
where
    Tail: Contains<T>,
{
    const VALUE: bool = true;
}

impl<Head, Tail: TypeArray, T> Contains<T> for TArr<Head, Tail>
where
    Tail: Contains<T>,
{
    const VALUE: bool = <Tail as Contains<T>>::VALUE;
}
*/

// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    type Arr = TArr<u8, TArr<u16, TArr<u32, TTerm>>>;

    #[test]
    fn test_len() {
        assert_eq!(tarr_len::<Arr>(), 3);
    }

    #[test]
    fn test_get() {
        type Get0 = <Arr as Get<Zero>>::Output;
        type Get1 = <Arr as Get<Succ<Zero>>>::Output;
        type Get2 = <Arr as Get<Succ<Succ<Zero>>>>::Output;
        assert_eq!(std::any::type_name::<Get0>(), "u8");
        assert_eq!(std::any::type_name::<Get1>(), "u16");
        assert_eq!(std::any::type_name::<Get2>(), "u32");
    }

    #[test]
    fn test_set() {
        let arr = TArr {
            first: 5u8,
            rest: TArr {
                first: 10u16,
                rest: TArr {
                    first: 15u32,
                    rest: TTerm,
                },
            },
        };

        type new_arr = <Arr as Set<Succ<Zero>, u16>>::Output;

        assert_eq!(tarr_len::<new_arr>(), 3);
        static_assertions::assert_type_eq_all!(<new_arr as Get<Zero>>::Output, u8);
        static_assertions::assert_type_eq_all!(<new_arr as Get<Succ<Zero>>>::Output, u16);
        static_assertions::assert_type_eq_all!(<new_arr as Get<Succ<Succ<Zero>>>>::Output, u32);
    }
}

// TArr<TArr<TArr<Term, u32>, u16>, u8>
//                      0,    1,    2
