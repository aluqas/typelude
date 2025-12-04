//! **Function Definitions**
//!
//! 型レベル関数のマーカー型と関連トレイトを定義します。
//! 関数の実装（`Evaluable`）は各 `types/*` モジュールで行われます。

use crate::eval::{EApply, EApply2, ELit};

// =============================================================================
// EFunction Trait
// =============================================================================

/// 関数を表すトレイト
///
/// `EWhile` などで使用される、型から型への変換を表します。
pub trait EFunction<A> {
    type Output;
}

// ELit を自動的にアンラップ
impl<F, T> EFunction<ELit<T>> for F
where
    F: EFunction<T>,
{
    type Output = <F as EFunction<T>>::Output;
}

// =============================================================================
// Function Markers: Boolean Operations
// =============================================================================

/// NOT: !A
pub struct FNot;
/// AND: A && B
pub struct FAnd;
/// OR: A || B
pub struct FOr;
/// NAND: !(A && B)
pub struct FNand;
/// NOR: !(A || B)
pub struct FNor;
/// XOR: A ^ B
pub struct FXor;
/// XNOR: !(A ^ B)
pub struct FXnor;

// =============================================================================
// Function Markers: Array Operations
// =============================================================================

/// 配列の長さ
pub struct FLen;
/// 配列の先頭要素
pub struct FHead;
/// 配列の先頭以外
pub struct FTail;
/// 配列が空かどうか
pub struct FIsEmpty;
/// 配列のインデックスアクセス
pub struct FGet;
/// 配列の結合
pub struct FConcat;
/// 配列の末尾に追加
pub struct FAppend;
/// 配列の先頭に追加
pub struct FPrepend;
/// 配列に要素が含まれるか
pub struct FContains;

// =============================================================================
// Function Markers: Equality Operations
// =============================================================================

/// 型の等価性: A == B
pub struct FEq;
/// 型の非等価性: A != B
pub struct FNotEq;

// =============================================================================
// Legacy Aliases
// 後方互換性のため、EApply<F*, ...> への type alias を提供
// =============================================================================

// Boolean (1 arg)
pub type ENot<A> = EApply<FNot, A>;

// Boolean (2 args)
pub type EAnd<A, B> = EApply2<FAnd, A, B>;
pub type EOr<A, B> = EApply2<FOr, A, B>;
pub type ENand<A, B> = EApply2<FNand, A, B>;
pub type ENor<A, B> = EApply2<FNor, A, B>;
pub type EXor<A, B> = EApply2<FXor, A, B>;
pub type EXnor<A, B> = EApply2<FXnor, A, B>;

// Array (1 arg)
pub type ELen<A> = EApply<FLen, A>;
pub type EHead<A> = EApply<FHead, A>;
pub type ETail<A> = EApply<FTail, A>;
pub type EIsEmpty<A> = EApply<FIsEmpty, A>;

// Array (2 args)
pub type EGet<A, I> = EApply2<FGet, A, I>;
pub type EConcat<A, B> = EApply2<FConcat, A, B>;
pub type EAppend<A, E> = EApply2<FAppend, A, E>;
pub type EPrepend<E, A> = EApply2<FPrepend, E, A>;
pub type EContains<A, X> = EApply2<FContains, A, X>;

// Equality (2 args)
pub type EEq<A, B> = EApply2<FEq, A, B>;
pub type ENotEq<A, B> = EApply2<FNotEq, A, B>;
