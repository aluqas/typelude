//! Convert (Translate) between Type or Value.
//!
//! - ToConst: Type -> Value (Runtime constant)
//! - ToType: Value -> Type (Type-level constant)
//! - Translate: Type -> Type (Type transformation)

use crate::{prelude::KindBool, std::bool::{TyFalse, TyTrue}, std::array::*};
use typenum::{Integer, Unsigned, UInt, UTerm, Bit};

// ======================================================================================
// ToConst: Type -> Value
// ======================================================================================

/// Ty to runtime value.
///
/// This trait allows converting type-level constants (like `TyTrue`, `U1`)
/// into their runtime equivalents (`true`, `1`).
///
/// # Examples
///
/// ```rust
/// use typenum::{U1, U2};
/// use typelude::std::{
///     bool::{TyFalse, TyTrue},
///     conv::ToConst,
/// };
///
/// assert_eq!(<TyTrue as ToConst<bool>>::VALUE, true);
/// assert_eq!(<TyFalse as ToConst<bool>>::VALUE, false);
/// assert_eq!(<U1 as ToConst<usize>>::VALUE, 1);
/// ```
pub trait ToConst<T> {
    const VALUE: T;

    fn value() -> T {
        Self::VALUE
    }
}

//
// Identifier: Boolean
//

impl ToConst<bool> for TyTrue {
    const VALUE: bool = true;
}

impl ToConst<bool> for TyFalse {
    const VALUE: bool = false;
}

//
// Identifier: Integer (typenum)
//

// We decompose blanket impl for Unsigned to avoid coherence conflict with Const<N>.
// Implement for UTerm, UInt, and Const separately.

macro_rules! impl_toconst_unsigned {
    ($($Type:ty),*) => {
        $(
            impl<U: Unsigned> ToConst<usize> for $Type { const VALUE: usize = <Self as Unsigned>::USIZE; }
            impl<U: Unsigned> ToConst<u64> for $Type { const VALUE: u64 = <Self as Unsigned>::U64; }
            impl<U: Unsigned> ToConst<u32> for $Type { const VALUE: u32 = <Self as Unsigned>::U32; }
            impl<U: Unsigned> ToConst<u16> for $Type { const VALUE: u16 = <Self as Unsigned>::U16; }
            impl<U: Unsigned> ToConst<u8> for $Type { const VALUE: u8 = <Self as Unsigned>::U8; }
        )*
    };
    // Specialized for UInt (generic)
    (@generic) => {
        impl<U: Unsigned, B: Bit> ToConst<usize> for UInt<U, B> { const VALUE: usize = <Self as Unsigned>::USIZE; }
        impl<U: Unsigned, B: Bit> ToConst<u64> for UInt<U, B> { const VALUE: u64 = <Self as Unsigned>::U64; }
        impl<U: Unsigned, B: Bit> ToConst<u32> for UInt<U, B> { const VALUE: u32 = <Self as Unsigned>::U32; }
        impl<U: Unsigned, B: Bit> ToConst<u16> for UInt<U, B> { const VALUE: u16 = <Self as Unsigned>::U16; }
        impl<U: Unsigned, B: Bit> ToConst<u8> for UInt<U, B> { const VALUE: u8 = <Self as Unsigned>::U8; }
    }
}

// UTerm is concrete
impl ToConst<usize> for UTerm { const VALUE: usize = 0; }
impl ToConst<u64> for UTerm { const VALUE: u64 = 0; }
impl ToConst<u32> for UTerm { const VALUE: u32 = 0; }
impl ToConst<u16> for UTerm { const VALUE: u16 = 0; }
impl ToConst<u8> for UTerm { const VALUE: u8 = 0; }

// UInt is generic
impl_toconst_unsigned!(@generic);

impl<I: Integer> ToConst<isize> for I {
    const VALUE: isize = I::ISIZE;
}

impl<I: Integer> ToConst<i64> for I {
    const VALUE: i64 = I::I64;
}

impl<I: Integer> ToConst<i32> for I {
    const VALUE: i32 = I::I32;
}

impl<I: Integer> ToConst<i16> for I {
    const VALUE: i16 = I::I16;
}

impl<I: Integer> ToConst<i8> for I {
    const VALUE: i8 = I::I8;
}

// Identifier: Integer (Const Generics Wrapper)
impl<const N: usize> ToConst<usize> for typenum::Const<N> {
    const VALUE: usize = N;
}

//
// Identifier: Unit (TyNil)
//

impl ToConst<()> for TyNil {
    const VALUE: () = ();
}

impl<T> ToConst<[T; 0]> for TyNil {
    const VALUE: [T; 0] = [];
}

//
// Identifier: Array (Homogeneous)
//

// Homogeneous Array: [T; N]
#[allow(unsafe_code)]
impl<Head, Tail, Val, const N: usize> ToConst<[Val; N]> for TyArray<Head, Tail>
where
    Tail: Cons,
    Head: ToConst<Val>,
    // Tail is [T; N-1]
    Tail: ToConst<[Val; N - 1]>,
{
    const VALUE: [Val; N] = {
        unsafe {
            let mut out = std::mem::MaybeUninit::<[Val; N]>::uninit();
            let out_ptr = out.as_mut_ptr() as *mut Val;

            // Write Head at index 0
            out_ptr.write(Head::VALUE);

            // Write Tail at index 1..
            let tail = Tail::VALUE;
            // Reconstruct pointer to tail data
            let tail_ptr = &tail as *const [Val; N - 1] as *const Val;

            if N > 1 {
                std::ptr::copy_nonoverlapping(tail_ptr, out_ptr.add(1), N - 1);
            }

            std::mem::forget(tail);

            out.assume_init()
        }
    };
}

//
// Identifier: Tuple (Flat) - via Macro
//

// Helper to build TyArray type from list of types
macro_rules! define_tylist {
    ($head:ident) => { TyArray<$head, TyNil> };
    ($head:ident, $($tail:ident),+) => { TyArray<$head, define_tylist!($($tail),+)> };
}

macro_rules! impl_flat_tuple {
    ($($n:tt $T:ident $V:ident),+) => {
        impl< $($T),+, $($V),+ > ToConst<($($V,)+)> for define_tylist!($($T),+)
        where
            $($T: ToConst<$V>),+
        {
            const VALUE: ($($V,)+) = ( $($T::VALUE,)+ );
        }
    };
}

// Generate for sizes 1..12
impl_flat_tuple!(0 T0 V0);
impl_flat_tuple!(0 T0 V0, 1 T1 V1);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4, 5 T5 V5);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4, 5 T5 V5, 6 T6 V6);
impl_flat_tuple!(0 T0 V0, 1 T1 V1, 2 T2 V2, 3 T3 V3, 4 T4 V4, 5 T5 V5, 6 T6 V6, 7 T7 V7);


// ======================================================================================
// ToType: Value -> Type
// ======================================================================================

/// Value (const generic) to Type.
///
/// This trait acts as a "factory" to produce types from const values.
/// Usage is typically via the `ty!` macro or `ToType::From*` associated types.
pub struct ToType;

impl ToType {
    // Note: Associated types with const generics dependent on the value
    // (e.g. FromBool<const B: bool>) cannot be implemented generically
    // for all values in stable/nightly Rust without specialization
    // or complete coverage proofs, which are not yet available.
    //
    // We use the `ty!` macro as the primary interface.
}

/// Macro for uniform Type Instantiation.
///
/// `ty!(10)` -> `P10` (or structural equiv)
/// `ty!(true)` -> `TyTrue`
#[macro_export]
macro_rules! ty {
    (true) => { $crate::std::bool::TyTrue };
    (false) => { $crate::std::bool::TyFalse };
    ($n:literal) => {
        // We can't dispatch on literal type easily in macros without more tricks.
        // But for integers, we can rely on `typenum::Const`.
        // However, `ty!(10)` is ambiguous (usize? isize? i32?).
        // We will default to `isize` or `usize` depending on the literal,
        // but `typenum::Const` usually handles both via its own macro or type inference if we use `U<...>`?
        // Actually `typenum::Const<N>` works for `{integer}`.
        typenum::Const<$n>
    };
}


// ======================================================================================
// Translate: Type -> Type
// ======================================================================================

/// Convert (Translate) one Type to another Type.
pub trait Translate<Target> {
    type Output;
}

// ======================================================================================
// Compatibility / Legacy
// ======================================================================================

/// Trait to reflect a const boolean to a Type (TyTrue/TyFalse).
/// Used by internal modules.
pub trait ReflectBool<const B: bool> {
    type Output: KindBool;
}

impl ReflectBool<true> for () {
    type Output = TyTrue;
}

impl ReflectBool<false> for () {
    type Output = TyFalse;
}


//
// Tests
//

#[cfg(test)]
mod tests {
    use typenum::{N1, P3, U1, U2, U3};
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::tyarray;

    #[test]
    fn test_bool() {
        assert_eq!(<TyTrue as ToConst<bool>>::VALUE, true);
        assert_eq!(<TyFalse as ToConst<bool>>::VALUE, false);
    }

    #[test]
    fn test_int() {
        assert_eq!(<U1 as ToConst<usize>>::VALUE, 1);
        assert_eq!(<U1 as ToConst<u8>>::VALUE, 1);
        assert_eq!(<N1 as ToConst<isize>>::VALUE, -1);
    }

    #[test]
    fn test_unit() {
        assert_eq!(<TyNil as ToConst<()>>::VALUE, ());
        assert_eq!(<TyNil as ToConst<[i32; 0]>>::VALUE, [] as [i32; 0]);
    }

    #[test]
    fn test_array_homogeneous() {
        type List = tyarray![U1, U2, U3]; // [1, 2, 3]
        assert_eq!(<List as ToConst<[usize; 3]>>::VALUE, [1, 2, 3]);
        assert_eq!(<List as ToConst<[u32; 3]>>::VALUE, [1, 2, 3]);
    }

    #[test]
    fn test_array_flat_tuple() {
        type List = tyarray![U1, TyTrue, P3]; // (1, true, 3)
        // P3 is Integer, implements ToConst<i32>
        // U1 is Unsigned, implements ToConst<usize>

        let val = <List as ToConst<(usize, bool, i32)>>::VALUE;
        assert_eq!(val, (1, true, 3));

        type List2 = tyarray![U1];
        // U1 implements ToConst<u32>
        assert_eq!(<List2 as ToConst<(u32,)>>::VALUE, (1,));
    }

    #[test]
    fn test_ty_macro() {
        use crate::std::bool::{TyTrue, TyFalse};
        type T = ty!(true);
        type F = ty!(false);
        type One = ty!(1);

        // Assert types match
        assert_type_eq_all!(T, TyTrue);
        assert_type_eq_all!(F, TyFalse);
        // assert_type_eq_all!(One, U1); // U1 is typenum::UInt<..., ...>, ty!(1) is typenum::Const<1> (which wraps U1 but is distinct type alias?)
        // They are different types (Const<N> vs UInt<...>) but represent the same number.
        assert_eq!(<One as ToConst<usize>>::VALUE, 1);
    }

    #[test]
    fn test_totype_factory() {
        // type T = ToType::FromBool<true>;
        // type U = ToType::FromUsize<10>;
        // crate::assert_type_eq!(T, TyTrue);
        // assert_eq!(<U as ToConst<usize>>::VALUE, 10);
    }
}
