//! WebAssembly ローカル変数アクセス命令。
//!
//! 関数引数・ローカル変数の読み書き。

use core::marker::PhantomData;

/// ローカル変数を読み込む。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLocalGet<Idx>(pub PhantomData<Idx>);

/// ローカル変数に書き込む（スタックトップを破棄）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLocalSet<Idx>(pub PhantomData<Idx>);

/// ローカル変数に書き込む（スタックトップ保持）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLocalTee<Idx>(pub PhantomData<Idx>);
