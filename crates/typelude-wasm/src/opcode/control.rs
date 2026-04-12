//! WebAssembly 制御フロー命令。
//!
//! 関数呼び出し（OpCall、OpCallIndirect）、
//! ブロック/ループ（OpBlock、OpLoop）、
//! 分岐（OpBr、OpBrif、OpBrTable）、復帰（OpReturn）。

use core::marker::PhantomData;

/// 関数直接呼び出し。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCall<Func>(PhantomData<Func>);

/// テーブル経由の間接呼び出し（型制約付き）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCallIndirect<TypeIdx, TableIdx = typenum::U0>(PhantomData<(TypeIdx, TableIdx)>);

/// ブロック構造（制御フロー標準化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBlock<Body>(PhantomData<Body>);

/// ループ構造（後方分岐）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLoop<Body>(PhantomData<Body>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBr<Depth>(PhantomData<Depth>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBrIf<Depth>(PhantomData<Depth>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpIf<Then, Else>(PhantomData<(Then, Else)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSelect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpReturn;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEndFunc;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEndBlock;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEndLoop;
