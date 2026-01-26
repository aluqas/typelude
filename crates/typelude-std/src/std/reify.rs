//! **Reification and Reflection**
//!
//! Provides traits to bridge the gap between type-level data and runtime
//! values.
//!
//! - **Reification**: Converts Type-level data to Runtime values (Type ->
//!   Value).
//! - **Reflection**: Converts Runtime constants to Type-level data (Value ->
//!   Type).

use typenum::{Bit, Integer, UInt, UTerm, Unsigned};

#[cfg(feature = "nightly")]
use crate::model::col::array::{Array, IsList};
use crate::{
    model::{
        col::array::Nil,
        prim::bool::{False, True},
    },
    std::traits::Bool,
};
/// Type to runtime value reification.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(True::reify(), true);
/// assert_eq!(U1::reify(), 1usize);
/// ```
pub trait Reify<T> {
    const REIFIED: T;

    fn reify() -> T {
        Self::REIFIED
    }
}

// ... Boolean ...
impl Reify<bool> for True {
    const REIFIED: bool = true;
}
impl Reify<bool> for False {
    const REIFIED: bool = false;
}

// ... Integer ...
macro_rules! impl_reify_unsigned {
    ($($Type:ty),*) => {
        $(
            impl<U: Unsigned> Reify<usize> for $Type { const REIFIED: usize = <Self as Unsigned>::USIZE; }
            impl<U: Unsigned> Reify<u64> for $Type { const REIFIED: u64 = <Self as Unsigned>::U64; }
            impl<U: Unsigned> Reify<u32> for $Type { const REIFIED: u32 = <Self as Unsigned>::U32; }
            impl<U: Unsigned> Reify<u16> for $Type { const REIFIED: u16 = <Self as Unsigned>::U16; }
            impl<U: Unsigned> Reify<u8> for $Type { const REIFIED: u8 = <Self as Unsigned>::U8; }
        )*
    };
    (@generic) => {
        impl<U: Unsigned, B: Bit> Reify<usize> for UInt<U, B> { const REIFIED: usize = <Self as Unsigned>::USIZE; }
        impl<U: Unsigned, B: Bit> Reify<u64> for UInt<U, B> { const REIFIED: u64 = <Self as Unsigned>::U64; }
        impl<U: Unsigned, B: Bit> Reify<u32> for UInt<U, B> { const REIFIED: u32 = <Self as Unsigned>::U32; }
        impl<U: Unsigned, B: Bit> Reify<u16> for UInt<U, B> { const REIFIED: u16 = <Self as Unsigned>::U16; }
        impl<U: Unsigned, B: Bit> Reify<u8> for UInt<U, B> { const REIFIED: u8 = <Self as Unsigned>::U8; }
    }
}

impl Reify<usize> for UTerm {
    const REIFIED: usize = 0;
}
impl Reify<u64> for UTerm {
    const REIFIED: u64 = 0;
}
impl Reify<u32> for UTerm {
    const REIFIED: u32 = 0;
}
impl Reify<u16> for UTerm {
    const REIFIED: u16 = 0;
}
impl Reify<u8> for UTerm {
    const REIFIED: u8 = 0;
}

impl_reify_unsigned!(@generic);

impl<I: Integer> Reify<isize> for I {
    const REIFIED: isize = I::ISIZE;
}
impl<I: Integer> Reify<i64> for I {
    const REIFIED: i64 = I::I64;
}
impl<I: Integer> Reify<i32> for I {
    const REIFIED: i32 = I::I32;
}
impl<I: Integer> Reify<i16> for I {
    const REIFIED: i16 = I::I16;
}
impl<I: Integer> Reify<i8> for I {
    const REIFIED: i8 = I::I8;
}

impl<const N: usize> Reify<usize> for typenum::Const<N> {
    const REIFIED: usize = N;
}

// ... Unit ...
impl Reify<()> for Nil {
    const REIFIED: () = ();
}
impl<T> Reify<[T; 0]> for Nil {
    const REIFIED: [T; 0] = [];
}

// ... Array ...
#[allow(unsafe_code)]
#[cfg(feature = "nightly")]
impl<Head, Tail, Val, const N: usize> Reify<[Val; N]> for Array<Head, Tail>
where
    Tail: IsList,
    Head: Reify<Val>,
    Tail: Reify<[Val; N - 1]>,
{
    const REIFIED: [Val; N] = {
        unsafe {
            let mut out = std::mem::MaybeUninit::<[Val; N]>::uninit();
            let out_ptr = out.as_mut_ptr() as *mut Val;
            out_ptr.write(Head::REIFIED);
            let tail = Tail::REIFIED;
            let tail_ptr = &tail as *const [Val; N - 1] as *const Val;
            if N > 1 {
                std::ptr::copy_nonoverlapping(tail_ptr, out_ptr.add(1), N - 1);
            }
            std::mem::forget(tail);
            out.assume_init()
        }
    };
}

// ... Tuple ...
macro_rules! define_tylist {
    ($head:ident) => { crate::model::col::array::Array<$head, Nil> };
    ($head:ident, $($tail:ident),+) => { crate::model::col::array::Array<$head, define_tylist!($($tail),+)> };
}
macro_rules! impl_flat_tuple {
    ($($n:tt $T:ident $V:ident),+) => {
        impl< $($T),+, $($V),+ > Reify<($($V,)+)> for define_tylist!($($T),+)
        where
            $($T: Reify<$V>),+
        {
            const REIFIED: ($($V,)+) = ( $($T::REIFIED,)+ );
        }
    };
}
impl_flat_tuple!(0 T0 V0);
impl_flat_tuple!(0 T0 V0, 1 T1 V1);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4, 5 T5 V5);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4, 5 T5 V5, 6 T6 V6);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4, 5 T5 V5, 6 T6 V6, 7 T7 V7);
/// Reflect a const boolean to a Type.
pub trait ReflectBool<const B: bool> {
    type Output: Bool;
}
impl ReflectBool<true> for () {
    type Output = True;
}
impl ReflectBool<false> for () {
    type Output = False;
}

/// Reflect a const usize to a Type.
pub trait ReflectUsize<const N: usize> {
    type Output;
}
/// Reflect a const isize to a Type.
pub trait ReflectInt<const N: isize> {
    type Output;
}

// Note: Implementation for integers typically requires macros or specific
// values if we want typenum U* For now, we can reflect to Const<N> or leave
// generic implementation for specialized crates to fill, or use typenum::Const
// as the output for generic integers.

impl<const N: usize> ReflectUsize<N> for () {
    type Output = typenum::Const<N>;
}
/*
impl<const N: isize> ReflectInt<N> for () {
    type Output = typenum::Const<N>; // Error: Const<N> requires N: usize
}
*/

/// Macro for value-to-type reflection.
#[macro_export]
macro_rules! ty {
    (true) => { <() as $crate::std::reify::ReflectBool<true>>::Output };
    (false) => { <() as $crate::std::reify::ReflectBool<false>>::Output };
    ($n:literal) => { typenum::Const<$n> }; // fallback or refined later
}
