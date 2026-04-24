//! WebAssembly メモリアクセス命令。
//!
//! リニアメモリ領域の読み書き（load/store）。
//! 8/16/32/64ビット単位でアクセス可能。

use core::marker::PhantomData;

use typenum::U0;

/// `i32.load`。
///
/// `MemArg` は alignment / offset / memory index を表します。
/// 現状の frontend / runtime 契約では memory index は `0` のみ対応です。
/// Stack effect: `[i32 addr] -> [i32 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.store`。
///
/// Stack effect: `[i32 addr, i32 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.load8_s`。
///
/// Stack effect: `[i32 addr] -> [i32 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load8S<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.load8_u`。
///
/// Stack effect: `[i32 addr] -> [i32 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load8U<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.load16_s`。
///
/// Stack effect: `[i32 addr] -> [i32 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load16S<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.load16_u`。
///
/// Stack effect: `[i32 addr] -> [i32 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load16U<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.store8`。
///
/// Stack effect: `[i32 addr, i32 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store8<MemArg = U0>(pub PhantomData<MemArg>);

/// `i32.store16`。
///
/// Stack effect: `[i32 addr, i32 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store16<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load8_s`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load8S<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load8_u`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load8U<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load16_s`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load16S<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load16_u`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load16U<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load32_s`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load32S<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load32_u`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load32U<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.load`。
///
/// Stack effect: `[i32 addr] -> [i64 value]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.store8`。
///
/// Stack effect: `[i32 addr, i64 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store8<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.store16`。
///
/// Stack effect: `[i32 addr, i64 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store16<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.store32`。
///
/// Stack effect: `[i32 addr, i64 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store32<MemArg = U0>(pub PhantomData<MemArg>);

/// `i64.store`。
///
/// Stack effect: `[i32 addr, i64 value] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store<MemArg = U0>(pub PhantomData<MemArg>);

/// `memory.size`。
///
/// `MemoryIdx` は現状 `0` のみ対応です。
/// Stack effect: `[] -> [i32 page_count]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpMemorySize<MemoryIdx = U0>(pub PhantomData<MemoryIdx>);

/// `memory.grow`。
///
/// `MemoryIdx` は現状 `0` のみ対応です。checked runtime でも trap
/// ではなく値を返します。 Stack effect: `[i32 delta_pages] -> [i32
/// previous_or_minus_one]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpMemoryGrow<MemoryIdx = U0>(pub PhantomData<MemoryIdx>);
