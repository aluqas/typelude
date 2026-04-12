//! WebAssembly グローバル変数アクセス命令。
//!
//! モジュールレベルのグローバル変数の読み書き。

use core::marker::PhantomData;

/// グローバル変数を読み込む。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGlobalGet<Idx>(pub PhantomData<Idx>);

/// グローバル変数に書き込む（可変グローバルのみ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGlobalSet<Idx>(pub PhantomData<Idx>);
