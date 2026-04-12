use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::Le;
use typenum::{B0, B1};

use crate::{
    helpers::{i32::TrueBit, name::NameEq},
    module::{
        HostFuncBinding, HostGlobalBinding, HostMemoryBinding, HostTableBinding, ImportGlobal,
        ImportMemory, ImportTable, NoLimit, WasmHostFunc,
    },
    state::{WasmGlobal, WasmMemory, WasmTable},
    value::{WasmI32, WasmI32Type, WasmI64, WasmI64Type},
};

pub struct MissingHostFuncBinding<ModuleName, FieldName>(pub PhantomData<(ModuleName, FieldName)>);
pub struct MissingHostGlobalBinding<ModuleName, FieldName>(
    pub PhantomData<(ModuleName, FieldName)>,
);
pub struct MissingHostMemoryBinding<ModuleName, FieldName>(
    pub PhantomData<(ModuleName, FieldName)>,
);
pub struct MissingHostTableBinding<ModuleName, FieldName>(
    pub PhantomData<(ModuleName, FieldName)>,
);
pub struct ResolvedHostFunc<Host>(pub PhantomData<Host>);

pub trait ResolveHostFuncBinding<ModuleName, FieldName> {
    type Output;
}

pub trait ResolveHostGlobalBinding<ModuleName, FieldName> {
    type Output;
}

pub trait ResolveHostMemoryBinding<ModuleName, FieldName> {
    type Output;
}

pub trait ResolveHostTableBinding<ModuleName, FieldName> {
    type Output;
}

#[doc(hidden)]
pub trait MatchFieldName<ExpectedField, BindingField> {
    type Output;
}

impl<ExpectedField, BindingField> MatchFieldName<ExpectedField, BindingField> for B0 {
    type Output = B0;
}

impl<ExpectedField, BindingField> MatchFieldName<ExpectedField, BindingField> for B1
where
    BindingField: NameEq<ExpectedField>,
{
    type Output = <BindingField as NameEq<ExpectedField>>::Output;
}

#[doc(hidden)]
pub trait ResolveHostFuncBindingHelper<ModuleName, FieldName, Host, Tail> {
    type Output;
}

impl<ModuleName, FieldName, Host, Tail>
    ResolveHostFuncBindingHelper<ModuleName, FieldName, Host, Tail> for B1
{
    type Output = ResolvedHostFunc<Host>;
}

impl<ModuleName, FieldName, Host, Tail>
    ResolveHostFuncBindingHelper<ModuleName, FieldName, Host, Tail> for B0
where
    Tail: ResolveHostFuncBinding<ModuleName, FieldName>,
{
    type Output = <Tail as ResolveHostFuncBinding<ModuleName, FieldName>>::Output;
}

impl<ModuleName, FieldName, BindingModule, BindingField, Host, Tail>
    ResolveHostFuncBinding<ModuleName, FieldName>
    for TArr<HostFuncBinding<BindingModule, BindingField, Host>, Tail>
where
    BindingModule: NameEq<ModuleName>,
    <BindingModule as NameEq<ModuleName>>::Output: MatchFieldName<FieldName, BindingField>,
    <<BindingModule as NameEq<ModuleName>>::Output as MatchFieldName<FieldName, BindingField>>::Output:
        ResolveHostFuncBindingHelper<ModuleName, FieldName, Host, Tail>,
{
    type Output = <<<
        BindingModule as NameEq<ModuleName>
    >::Output as MatchFieldName<FieldName, BindingField>>::Output as ResolveHostFuncBindingHelper<
        ModuleName,
        FieldName,
        Host,
        Tail,
    >>::Output;
}

impl<ModuleName, FieldName> ResolveHostFuncBinding<ModuleName, FieldName> for TTerm {
    type Output = MissingHostFuncBinding<ModuleName, FieldName>;
}

#[doc(hidden)]
pub trait ResolveHostGlobalBindingHelper<ModuleName, FieldName, Global, Tail> {
    type Output;
}

impl<ModuleName, FieldName, Global, Tail>
    ResolveHostGlobalBindingHelper<ModuleName, FieldName, Global, Tail> for B1
{
    type Output = Global;
}

impl<ModuleName, FieldName, Global, Tail>
    ResolveHostGlobalBindingHelper<ModuleName, FieldName, Global, Tail> for B0
where
    Tail: ResolveHostGlobalBinding<ModuleName, FieldName>,
{
    type Output = <Tail as ResolveHostGlobalBinding<ModuleName, FieldName>>::Output;
}

impl<ModuleName, FieldName, BindingModule, BindingField, Global, Tail>
    ResolveHostGlobalBinding<ModuleName, FieldName>
    for TArr<HostGlobalBinding<BindingModule, BindingField, Global>, Tail>
where
    BindingModule: NameEq<ModuleName>,
    <BindingModule as NameEq<ModuleName>>::Output: MatchFieldName<FieldName, BindingField>,
    <<BindingModule as NameEq<ModuleName>>::Output as MatchFieldName<FieldName, BindingField>>::Output:
        ResolveHostGlobalBindingHelper<ModuleName, FieldName, Global, Tail>,
{
    type Output = <<<
        BindingModule as NameEq<ModuleName>
    >::Output as MatchFieldName<FieldName, BindingField>>::Output as ResolveHostGlobalBindingHelper<
        ModuleName,
        FieldName,
        Global,
        Tail,
    >>::Output;
}

impl<ModuleName, FieldName> ResolveHostGlobalBinding<ModuleName, FieldName> for TTerm {
    type Output = MissingHostGlobalBinding<ModuleName, FieldName>;
}

#[doc(hidden)]
pub trait ResolveHostMemoryBindingHelper<ModuleName, FieldName, Memory, Tail> {
    type Output;
}

impl<ModuleName, FieldName, Memory, Tail>
    ResolveHostMemoryBindingHelper<ModuleName, FieldName, Memory, Tail> for B1
{
    type Output = Memory;
}

impl<ModuleName, FieldName, Memory, Tail>
    ResolveHostMemoryBindingHelper<ModuleName, FieldName, Memory, Tail> for B0
where
    Tail: ResolveHostMemoryBinding<ModuleName, FieldName>,
{
    type Output = <Tail as ResolveHostMemoryBinding<ModuleName, FieldName>>::Output;
}

impl<ModuleName, FieldName, BindingModule, BindingField, Memory, Tail>
    ResolveHostMemoryBinding<ModuleName, FieldName>
    for TArr<HostMemoryBinding<BindingModule, BindingField, Memory>, Tail>
where
    BindingModule: NameEq<ModuleName>,
    <BindingModule as NameEq<ModuleName>>::Output: MatchFieldName<FieldName, BindingField>,
    <<BindingModule as NameEq<ModuleName>>::Output as MatchFieldName<FieldName, BindingField>>::Output:
        ResolveHostMemoryBindingHelper<ModuleName, FieldName, Memory, Tail>,
{
    type Output = <<<
        BindingModule as NameEq<ModuleName>
    >::Output as MatchFieldName<FieldName, BindingField>>::Output as ResolveHostMemoryBindingHelper<
        ModuleName,
        FieldName,
        Memory,
        Tail,
    >>::Output;
}

impl<ModuleName, FieldName> ResolveHostMemoryBinding<ModuleName, FieldName> for TTerm {
    type Output = MissingHostMemoryBinding<ModuleName, FieldName>;
}

#[doc(hidden)]
pub trait ResolveHostTableBindingHelper<ModuleName, FieldName, Table, Tail> {
    type Output;
}

impl<ModuleName, FieldName, Table, Tail>
    ResolveHostTableBindingHelper<ModuleName, FieldName, Table, Tail> for B1
{
    type Output = Table;
}

impl<ModuleName, FieldName, Table, Tail>
    ResolveHostTableBindingHelper<ModuleName, FieldName, Table, Tail> for B0
where
    Tail: ResolveHostTableBinding<ModuleName, FieldName>,
{
    type Output = <Tail as ResolveHostTableBinding<ModuleName, FieldName>>::Output;
}

impl<ModuleName, FieldName, BindingModule, BindingField, Table, Tail>
    ResolveHostTableBinding<ModuleName, FieldName>
    for TArr<HostTableBinding<BindingModule, BindingField, Table>, Tail>
where
    BindingModule: NameEq<ModuleName>,
    <BindingModule as NameEq<ModuleName>>::Output: MatchFieldName<FieldName, BindingField>,
    <<BindingModule as NameEq<ModuleName>>::Output as MatchFieldName<FieldName, BindingField>>::Output:
        ResolveHostTableBindingHelper<ModuleName, FieldName, Table, Tail>,
{
    type Output = <<<
        BindingModule as NameEq<ModuleName>
    >::Output as MatchFieldName<FieldName, BindingField>>::Output as ResolveHostTableBindingHelper<
        ModuleName,
        FieldName,
        Table,
        Tail,
    >>::Output;
}

impl<ModuleName, FieldName> ResolveHostTableBinding<ModuleName, FieldName> for TTerm {
    type Output = MissingHostTableBinding<ModuleName, FieldName>;
}

pub trait ImportedFuncCompat<ImportKind> {
    type Output;
}

impl<FuncType, Host> ImportedFuncCompat<crate::module::ImportFunc<FuncType>>
    for ResolvedHostFunc<Host>
{
    type Output = WasmHostFunc<FuncType, Host>;
}

pub trait ImportedGlobalCompat<ImportKind> {
    type Output;
}

impl<Mutability, ValueT> ImportedGlobalCompat<ImportGlobal<Mutability, WasmI32Type>>
    for WasmGlobal<Mutability, WasmI32<ValueT>>
{
    type Output = WasmGlobal<Mutability, WasmI32<ValueT>>;
}

impl<Mutability, ValueT> ImportedGlobalCompat<ImportGlobal<Mutability, WasmI64Type>>
    for WasmGlobal<Mutability, WasmI64<ValueT>>
{
    type Output = WasmGlobal<Mutability, WasmI64<ValueT>>;
}

pub trait ImportMaxCompat<ActualMax> {}

impl<ActualMax> ImportMaxCompat<ActualMax> for NoLimit {}

impl<ExpectedMax, ActualMax> ImportMaxCompat<ActualMax> for ExpectedMax
where
    ExpectedMax: typenum::Unsigned,
    ActualMax: Le<ExpectedMax>,
    <ActualMax as Le<ExpectedMax>>::Output: TrueBit,
{
}

pub trait ImportedMemoryCompat<ImportKind> {
    type Output;
}

impl<MinPages, MaxPages, Pages, ActualMax, Cells>
    ImportedMemoryCompat<ImportMemory<MinPages, MaxPages>> for WasmMemory<Pages, ActualMax, Cells>
where
    MinPages: Le<Pages>,
    <MinPages as Le<Pages>>::Output: TrueBit,
    MaxPages: ImportMaxCompat<ActualMax>,
{
    type Output = WasmMemory<Pages, ActualMax, Cells>;
}

pub trait ImportedTableCompat<ImportKind> {
    type Output;
}

impl<Min, Max, ActualMin, ActualMax, Entries> ImportedTableCompat<ImportTable<Min, Max>>
    for WasmTable<ActualMin, ActualMax, Entries>
where
    Min: Le<ActualMin>,
    <Min as Le<ActualMin>>::Output: TrueBit,
    Max: ImportMaxCompat<ActualMax>,
{
    type Output = WasmTable<ActualMin, ActualMax, Entries>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::*;

    include!("../tests/cases/helpers_import.rs");
}
