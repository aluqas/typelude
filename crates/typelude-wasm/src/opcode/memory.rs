//! WebAssembly メモリアクセス命令。
//!
//! リニアメモリ領域の読み書き（load/store）。
//! 8/16/32/64ビット単位でアクセス可能。

use core::marker::PhantomData;

use typenum::U0;

/// メモリから i32 をロード。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load<MemArg = U0>(pub PhantomData<MemArg>);

/// メモリに i32 をストア。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store<MemArg = U0>(pub PhantomData<MemArg>);

/// メモリから i32 をロード（8ビット符号拡張）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load8S<MemArg = U0>(pub PhantomData<MemArg>);

/// メモリから i32 をロード（8ビット符号なし拡張）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load8U<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load16S<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load16U<MemArg = U0>(pub PhantomData<MemArg>);

/// メモリに i32 をストア（8ビット）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store8<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store16<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load8S<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load8U<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load16S<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load16U<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load32S<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load32U<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store8<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store16<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store32<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpMemorySize<MemoryIdx = U0>(pub PhantomData<MemoryIdx>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpMemoryGrow<MemoryIdx = U0>(pub PhantomData<MemoryIdx>);
