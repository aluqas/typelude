//! WebAssembly リテラル定数命令。
//!
//! i32/i64 の即値をスタックプッシュ。

use core::marker::PhantomData;

/// i32 定数をスタックへプッシュ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Const<Val>(PhantomData<Val>);

/// i64 定数をスタックへプッシュ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Const<Val>(PhantomData<Val>);
