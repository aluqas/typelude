//! WebAssembly 制御フロー命令。
//!
//! 関数呼び出し（OpCall、OpCallIndirect）、
//! ブロック/ループ（OpBlock、OpLoop）、
//! 分岐（OpBr、OpBrif、OpBrTable）、復帰（OpReturn）。

use core::marker::PhantomData;

/// `nop`。
///
/// Stack effect: `[] -> []`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpNop;

/// `unreachable`。
///
/// checked runtime では `TrapUnreachable` を返します。
/// success-only runtime では通常の成功 step としては扱えません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpUnreachable;

/// `call`。
///
/// `Func` は module function space 内の関数 index です。
/// Stack effect は対象関数型に従い、params を pop して results を push します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCall<Func>(PhantomData<Func>);

/// `call_indirect`。
///
/// `TypeIdx` は期待関数型、`TableIdx` は参照する table index です。
/// checked runtime では null / table OOB / type mismatch を trap 化します。
/// Stack effect: `[i32 table_slot, params...] -> [results...]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCallIndirect<TypeIdx, TableIdx = typenum::U0>(PhantomData<(TypeIdx, TableIdx)>);

/// `block`。
///
/// `Body` は block 本体の命令列です。
/// 実行時には branch stack に block continuation を積み、body の末尾に
/// synthetic end opcode を追加します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBlock<Body>(PhantomData<Body>);

/// `loop`。
///
/// `Body` は loop 本体の命令列です。
/// `br` が loop を指す場合は loop body へ戻る continuation として扱われます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLoop<Body>(PhantomData<Body>);

/// `br`。
///
/// `Depth` は branch stack に対する相対深さです。
/// Stack effect は target block / loop の仕様に依存します。この実装では現在
/// stack をそのまま遷移先へ渡します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBr<Depth>(PhantomData<Depth>);

/// `br_if`。
///
/// `Depth` は branch stack に対する相対深さです。
/// Stack effect: `[i32 cond] -> []` plus branch transition。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBrIf<Depth>(PhantomData<Depth>);

/// `br_table`。
///
/// `Targets` は分岐先一覧、`Default` は範囲外 index 用の分岐先です。
/// Stack effect: `[i32 index] -> []` plus selected branch transition。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpBrTable<Targets, Default>(PhantomData<(Targets, Default)>);

/// `if`。
///
/// `Then` と `Else` は各分岐本体の命令列です。
/// Stack effect: `[i32 cond] -> []` plus selected branch program。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpIf<Then, Else>(PhantomData<(Then, Else)>);

/// `select`。
///
/// 現在の frontend では typed select は `i32` / `i64` に制限されます。
/// Stack effect: `[T false_value, T true_value, i32 cond] -> [T selected]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSelect;

/// `return`。
///
/// Stack effect は呼び出し元 frame への復帰として扱われます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpReturn;

/// 関数終端を表す内部 synthetic opcode。
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEndFunc;

/// block 終端を表す内部 synthetic opcode。
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEndBlock;

/// loop 終端を表す内部 synthetic opcode。
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEndLoop;
