//! **Type-Level Stack Machine Instruction Definitions**
//!
//! スタックマシンの命令セットを定義します。

use std::marker::PhantomData;
use crate::eval::Sealed;

/// 命令マーカー
pub trait Instruction: Sealed {}

/// Push N: スタックに値 N を積む
#[derive(Debug, Clone, Copy)]
pub struct OpPush<N>(PhantomData<N>);
impl<N> Sealed for OpPush<N> {}
impl<N> Instruction for OpPush<N> {}

/// Add: スタックの上位2つを取り出し、足して積む
#[derive(Debug, Clone, Copy)]
pub struct OpAdd;
impl Sealed for OpAdd {}
impl Instruction for OpAdd {}

/// Sub: スタックの上位2つを取り出し、引いて積む
#[derive(Debug, Clone, Copy)]
pub struct OpSub;
impl Sealed for OpSub {}
impl Instruction for OpSub {}

/// Dup: スタックの先頭要素を複製する
#[derive(Debug, Clone, Copy)]
pub struct OpDup;
impl Sealed for OpDup {}
impl Instruction for OpDup {}

/// Swap: スタックの先頭2つの要素を入れ替える
#[derive(Debug, Clone, Copy)]
pub struct OpSwap;
impl Sealed for OpSwap {}
impl Instruction for OpSwap {}

/// Drop: スタックの先頭要素を破棄する
#[derive(Debug, Clone, Copy)]
pub struct OpDrop;
impl Sealed for OpDrop {}
impl Instruction for OpDrop {}

/// If: スタックの先頭がTrueならThen、FalseならElseを実行する
/// - ThenProg: Trueの場合に実行する命令列
/// - ElseProg: Falseの場合に実行する命令列
#[derive(Debug, Clone, Copy)]
pub struct OpIf<ThenProg, ElseProg>(PhantomData<(ThenProg, ElseProg)>);
impl<T, E> Sealed for OpIf<T, E> {}
impl<T, E> Instruction for OpIf<T, E> {}

/// Load: スタックからアドレスをポップし、メモリから値を読み出してプッシュする
#[derive(Debug, Clone, Copy)]
pub struct OpLoad;
impl Sealed for OpLoad {}
impl Instruction for OpLoad {}

/// Store: スタックから値とアドレスをポップし、メモリに値を書き込む
/// [Value, Addr, ...] -> Memory[Addr] = Value
#[derive(Debug, Clone, Copy)]
pub struct OpStore;
impl Sealed for OpStore {}
impl Instruction for OpStore {}

/// Call: 関数呼び出し
/// 現在の残りのプログラム（継続）をコールスタックに積み、TargetProgにジャンプする
#[derive(Debug, Clone, Copy)]
pub struct OpCall<TargetProg>(PhantomData<TargetProg>);
impl<TargetProg> Sealed for OpCall<TargetProg> {}
impl<TargetProg> Instruction for OpCall<TargetProg> {}

/// Return: 関数から復帰
/// コールスタックから継続を取り出し、そこへジャンプする
#[derive(Debug, Clone, Copy)]
pub struct OpReturn;
impl Sealed for OpReturn {}
impl Instruction for OpReturn {}

/// Eq: スタックの上位2つが等しいか判定 (A == B)
#[derive(Debug, Clone, Copy)]
pub struct OpEq;
impl Sealed for OpEq {}
impl Instruction for OpEq {}

/// Neq: スタックの上位2つが等しくないか判定 (A != B)
#[derive(Debug, Clone, Copy)]
pub struct OpNeq;
impl Sealed for OpNeq {}
impl Instruction for OpNeq {}

/// Lt: スタックの上位2つを比較 (A < B)
/// Stack: [B, A, ...] -> Push (A < B)
#[derive(Debug, Clone, Copy)]
pub struct OpLt;
impl Sealed for OpLt {}
impl Instruction for OpLt {}

/// Gt: スタックの上位2つを比較 (A > B)
/// Stack: [B, A, ...] -> Push (A > B)
#[derive(Debug, Clone, Copy)]
pub struct OpGt;
impl Sealed for OpGt {}
impl Instruction for OpGt {}

/// Not: 論理否定 (!A)
#[derive(Debug, Clone, Copy)]
pub struct OpNot;
impl Sealed for OpNot {}
impl Instruction for OpNot {}

/// And: 論理積 (A && B)
#[derive(Debug, Clone, Copy)]
pub struct OpAnd;
impl Sealed for OpAnd {}
impl Instruction for OpAnd {}

/// Or: 論理和 (A || B)
#[derive(Debug, Clone, Copy)]
pub struct OpOr;
impl Sealed for OpOr {}
impl Instruction for OpOr {}

/// While: ループ構造
/// 1. CondProgを実行
/// 2. スタックトップがTrueならBodyProgを実行し、1に戻る
/// 3. Falseなら終了（ループを抜ける）
#[derive(Debug, Clone, Copy)]
pub struct OpWhile<CondProg, BodyProg>(PhantomData<(CondProg, BodyProg)>);
impl<C, B> Sealed for OpWhile<C, B> {}
impl<C, B> Instruction for OpWhile<C, B> {}
