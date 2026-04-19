//! 関数呼び出しの helper 群。
//!
//! `OpCall` / `OpCallIndirect` の意味論は、関数 index lookup、parameter pop、
//! local 初期化、host call 結果の state
//! 化に分解されています。このファイルはその分解点を trait family
//! として提供します。
//!
//! 型不一致は基本的に trait 未解決として失敗します。checked `call_indirect`
//! では `FuncTypeEq` の結果を使い、型不一致を `TrapCallIndirectTypeMismatch`
//! に変換します。

use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{And, Append, Concat, Get};
use typenum::{B0, B1, U0};

use crate::{
    func::WasmFunc,
    module::{WasmFuncType, WasmHostFunc, WasmResolvedModule},
    value::{
        WasmF32, WasmF32Type, WasmF64, WasmF64Type, WasmI32, WasmI32Type, WasmI64, WasmI64Type,
    },
};

/// 型リストを反転する helper trait。
///
/// 関数引数は stack から逆順に取り出すため、parameter list の整形に使います。
pub trait ReverseList {
    type Output;
}

impl ReverseList for TTerm {
    type Output = TTerm;
}

impl<Head, Tail> ReverseList for TArr<Head, Tail>
where
    Tail: ReverseList,
    <Tail as ReverseList>::Output: Append<Head>,
{
    type Output = <<Tail as ReverseList>::Output as Append<Head>>::Output;
}

/// stack のトップから期待値型に一致する 1 引数を取り出す helper。
///
/// 不一致時は trait 未解決として失敗します。
pub trait PopArg<ExpectedType> {
    type RemainingStack;
    type Value;
}

impl<ValueT, Tail> PopArg<WasmI32Type> for TArr<WasmI32<ValueT>, Tail> {
    type RemainingStack = Tail;
    type Value = WasmI32<ValueT>;
}

impl<ValueT, Tail> PopArg<WasmI64Type> for TArr<WasmI64<ValueT>, Tail> {
    type RemainingStack = Tail;
    type Value = WasmI64<ValueT>;
}

impl<ValueT, Tail> PopArg<WasmF32Type> for TArr<WasmF32<ValueT>, Tail> {
    type RemainingStack = Tail;
    type Value = WasmF32<ValueT>;
}

impl<ValueT, Tail> PopArg<WasmF64Type> for TArr<WasmF64<ValueT>, Tail> {
    type RemainingStack = Tail;
    type Value = WasmF64<ValueT>;
}

/// stack から複数引数を関数シグネチャに従って取り出す helper。
pub trait PopArgs<ParamTypes> {
    type RemainingStack;
    type Params;
}

impl<Stack> PopArgs<TTerm> for Stack {
    type RemainingStack = Stack;
    type Params = TTerm;
}

impl<Stack, ParamType, TailTypes> PopArgs<TArr<ParamType, TailTypes>> for Stack
where
    Stack: PopArg<ParamType>,
    <Stack as PopArg<ParamType>>::RemainingStack: PopArgs<TailTypes>,
    <<Stack as PopArg<ParamType>>::RemainingStack as PopArgs<TailTypes>>::Params:
        Append<<Stack as PopArg<ParamType>>::Value>,
{
    type RemainingStack =
        <<Stack as PopArg<ParamType>>::RemainingStack as PopArgs<TailTypes>>::RemainingStack;
    type Params =
        <<<Stack as PopArg<ParamType>>::RemainingStack as PopArgs<TailTypes>>::Params as Append<
            <Stack as PopArg<ParamType>>::Value,
        >>::Output;
}

/// 関数型から parameter list を抽出する helper。
pub trait ParamTypes {
    type Output;
}

impl<Params, Results> ParamTypes for WasmFuncType<Params, Results> {
    type Output = Params;
}

/// stack 上の引数と local 宣言から実行時 locals 配列を構築する helper。
///
/// WASM の operand stack は top-first なので、parameter list を反転してから pop
/// します。 pop した params と zero-initialized local 宣言列を連結したものが
/// current locals になります。
pub trait BindLocals<FuncType, LocalInits> {
    type Output;
}

/// 値型に対応する zero 値を与える helper。
pub trait ZeroValueForType {
    type Output;
}

impl ZeroValueForType for WasmI32Type {
    type Output = WasmI32<U0>;
}

impl ZeroValueForType for WasmI64Type {
    type Output = WasmI64<U0>;
}

impl ZeroValueForType for WasmF32Type {
    type Output = WasmF32<U0>;
}

impl ZeroValueForType for WasmF64Type {
    type Output = WasmF64<U0>;
}

/// local 宣言列を zero-initialized な local 値列へ materialize する helper。
pub trait MaterializeLocals {
    type Output;
}

impl MaterializeLocals for TTerm {
    type Output = TTerm;
}

impl<Head, Tail> MaterializeLocals for TArr<Head, Tail>
where
    Head: ZeroValueForType,
    Tail: MaterializeLocals,
{
    type Output = TArr<<Head as ZeroValueForType>::Output, <Tail as MaterializeLocals>::Output>;
}

impl<Stack, FuncType, LocalDecls> BindLocals<FuncType, LocalDecls> for Stack
where
    FuncType: ParamTypes,
    <FuncType as ParamTypes>::Output: ReverseList,
    Stack: PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>,
    LocalDecls: MaterializeLocals,
    <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params:
        Concat<<LocalDecls as MaterializeLocals>::Output>,
{
    type Output = <<Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params as Concat<
        <LocalDecls as MaterializeLocals>::Output,
    >>::Output;
}

/// module の function space から関数 index を引く helper。
pub trait ModuleFuncLookup<FuncIdx> {
    type Output;
}

impl<Funcs, Types, Exports, FuncIdx> ModuleFuncLookup<FuncIdx>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    Funcs: Get<FuncIdx>,
{
    type Output = <Funcs as Get<FuncIdx>>::Output;
}

/// 関数定義から `WasmFuncType` を取り出す helper。
pub trait FuncSignature {
    type Output;
}

impl<FuncType, LocalDecls, Program> FuncSignature for WasmFunc<FuncType, LocalDecls, Program> {
    type Output = FuncType;
}

impl<FuncType, Host> FuncSignature for WasmHostFunc<FuncType, Host> {
    type Output = FuncType;
}

/// 値型どうしの等価性判定。
///
/// `B1` が一致、`B0` が不一致です。
pub trait ValTypeEq<Rhs> {
    type Output;
}

macro_rules! impl_val_type_eq {
    ($lhs:ty, $rhs:ty => $out:ty) => {
        impl ValTypeEq<$rhs> for $lhs {
            type Output = $out;
        }
    };
}

impl_val_type_eq!(WasmI32Type, WasmI32Type => B1);
impl_val_type_eq!(WasmI32Type, WasmI64Type => B0);
impl_val_type_eq!(WasmI32Type, WasmF32Type => B0);
impl_val_type_eq!(WasmI32Type, WasmF64Type => B0);
impl_val_type_eq!(WasmI64Type, WasmI32Type => B0);
impl_val_type_eq!(WasmI64Type, WasmI64Type => B1);
impl_val_type_eq!(WasmI64Type, WasmF32Type => B0);
impl_val_type_eq!(WasmI64Type, WasmF64Type => B0);
impl_val_type_eq!(WasmF32Type, WasmI32Type => B0);
impl_val_type_eq!(WasmF32Type, WasmI64Type => B0);
impl_val_type_eq!(WasmF32Type, WasmF32Type => B1);
impl_val_type_eq!(WasmF32Type, WasmF64Type => B0);
impl_val_type_eq!(WasmF64Type, WasmI32Type => B0);
impl_val_type_eq!(WasmF64Type, WasmI64Type => B0);
impl_val_type_eq!(WasmF64Type, WasmF32Type => B0);
impl_val_type_eq!(WasmF64Type, WasmF64Type => B1);

/// 値型リストどうしの等価性判定。
pub trait ValListEq<Rhs> {
    type Output;
}

impl ValListEq<TTerm> for TTerm {
    type Output = B1;
}

impl<Head, Tail> ValListEq<TTerm> for TArr<Head, Tail> {
    type Output = B0;
}

impl<Head, Tail> ValListEq<TArr<Head, Tail>> for TTerm {
    type Output = B0;
}

impl<Head, Tail, RhsHead, RhsTail> ValListEq<TArr<RhsHead, RhsTail>> for TArr<Head, Tail>
where
    Head: ValTypeEq<RhsHead>,
    Tail: ValListEq<RhsTail>,
    <Head as ValTypeEq<RhsHead>>::Output: And<<Tail as ValListEq<RhsTail>>::Output>,
{
    type Output = <<Head as ValTypeEq<RhsHead>>::Output as And<
        <Tail as ValListEq<RhsTail>>::Output,
    >>::Output;
}

/// 関数型どうしの等価性判定。
///
/// checked `call_indirect` の型一致判定に使います。
pub trait FuncTypeEq<Rhs> {
    type Output;
}

impl<Params, Results, RhsParams, RhsResults> FuncTypeEq<WasmFuncType<RhsParams, RhsResults>>
    for WasmFuncType<Params, Results>
where
    Params: ValListEq<RhsParams>,
    Results: ValListEq<RhsResults>,
    <Params as ValListEq<RhsParams>>::Output: And<<Results as ValListEq<RhsResults>>::Output>,
{
    type Output = <<Params as ValListEq<RhsParams>>::Output as And<
        <Results as ValListEq<RhsResults>>::Output,
    >>::Output;
}

/// ホスト関数呼び出し結果。
///
/// `Store` は更新後 store、`Results` は戻り値列です。
#[derive(Debug, Default)]
pub struct HostCallResult<Store, Results>(pub PhantomData<(Store, Results)>);

/// ホスト関数の型レベル実装インターフェース。
///
/// `FuncType` と `Args` に従って呼び出され、`HostCallResult<Store, Results>`
/// を返すことを期待します。
pub trait HostCall<FuncType, Store, Args> {
    /// 呼び出し結果。通常は `HostCallResult<Store, Results>`。
    type Output;
}
