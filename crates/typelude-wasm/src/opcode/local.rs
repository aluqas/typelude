//! WebAssembly ローカル変数アクセス命令。
//!
//! 関数引数・ローカル変数の読み書き。

use core::marker::PhantomData;

/// ローカル変数を読み込む。
///
/// Stack effect: `[] -> [T]`。
/// `Idx` は現在関数 frame の local index です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLocalGet<Idx>(pub PhantomData<Idx>);

/// ローカル変数に書き込む（スタックトップを破棄）。
///
/// Stack effect: `[T] -> []`。
/// `Idx` が指す local の型は stack top の値型で置き換えられます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLocalSet<Idx>(pub PhantomData<Idx>);

/// ローカル変数に書き込む（スタックトップ保持）。
///
/// Stack effect: `[T] -> [T]`。
/// `local.set` と異なり、書き込んだ値を operand stack 上にも残します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLocalTee<Idx>(pub PhantomData<Idx>);
