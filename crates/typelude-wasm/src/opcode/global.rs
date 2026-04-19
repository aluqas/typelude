//! WebAssembly グローバル変数アクセス命令。
//!
//! モジュールレベルのグローバル変数の読み書き。

use core::marker::PhantomData;

/// グローバル変数を読み込む。
///
/// Stack effect: `[] -> [T]`。
/// `Idx` は module instance の global index です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGlobalGet<Idx>(pub PhantomData<Idx>);

/// グローバル変数に書き込む（可変グローバルのみ）。
///
/// Stack effect: `[T] -> []`。
/// `GlobalMut` にだけ実装され、`GlobalConst` への書き込みは trait
/// 未解決になります。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGlobalSet<Idx>(pub PhantomData<Idx>);
