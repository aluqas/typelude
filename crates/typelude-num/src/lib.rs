//! 型レベル数値プリミティブと算術演算。
//!
//! ## 責務
//!
//! ペアノ数（Zero、Succ）による型レベル自然数と
//! 算術能力トレイト（Add、Sub、Mul、Div）を提供。
//! typenum を再エクスポート（相互運用性確保）。
//!
//! ## 名前空間マッピング
//!
//! - `Add` -> `OpAdd`
//! - `Sub` -> `OpSub`
//! - `Mul` -> `OpMul`
//! - `Div` -> `OpDiv`
//! - `Rem` -> `OpRem`
//! - `Pow` -> `OpPow`

#![recursion_limit = "256"]

pub use typelude_std::core::{Add, Div, Mul, Pow, Rem, Sub};
pub use typenum::*;
extern crate typenum;

pub mod peano;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{Apply, EAdd, EPow, ERem, Evaluate, OpPow, OpRem};

    use super::{U1, U2, U3, U5, U8};

    #[test]
    fn typenum_interop_works_through_canonical_surface() {
        assert_type_eq_all!(Evaluate<EAdd<U2, U3>>, U5);
        assert_type_eq_all!(Evaluate<ERem<U5, U2>>, U1);
        assert_type_eq_all!(Evaluate<EPow<U2, U3>>, U8);
        assert_type_eq_all!(Evaluate<Apply<OpRem, (U5, U2)>>, U1);
        assert_type_eq_all!(Evaluate<Apply<OpPow, (U2, U3)>>, U8);
    }
}
