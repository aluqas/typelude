//! WebAssembly リテラル定数命令。
//!
//! i32/i64 の即値をスタックプッシュ。

use core::marker::PhantomData;

/// `i32.const`。
///
/// Stack effect: `[] -> [i32]`。
/// `Val` は 32bit bit-pattern を typenum で表した値です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Const<Val>(PhantomData<Val>);

/// `i64.const`。
///
/// Stack effect: `[] -> [i64]`。
/// `Val` は 64bit bit-pattern を typenum で表した値です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Const<Val>(PhantomData<Val>);
