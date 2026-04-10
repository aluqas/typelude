use typelude_col::{TArr, TTerm};
use typelude_std::core::{Append, Get};

use crate::{
    helpers::{
        i32::U4294967295,
        memory::MemoryWriteBytes,
        table::TableWriteRefs,
    },
    module::{
        GlobalConst, InitGlobalGet, InitI32Const, NoLimit, WasmDataSegment, WasmElemSegment,
        WasmFuncSpace, WasmGlobalDecl, WasmHostEnv, WasmInstance, WasmMemoryDecl, WasmModule,
        WasmResolvedModule, WasmTableDecl,
    },
    state::{WasmGlobal, WasmMemory, WasmStore, WasmTable},
    value::WasmI32,
};

pub trait Instantiate<Env> {
    type Output;
}

pub trait ToStoreMax {
    type Output;
}

impl ToStoreMax for NoLimit {
    type Output = U4294967295;
}

impl<MaxPages> ToStoreMax for MaxPages
where
    MaxPages: typenum::Unsigned,
{
    type Output = MaxPages;
}

pub trait ResolveInitExprValue<Expr, Globals> {
    type Output;
}

impl<ValueT, Globals> ResolveInitExprValue<InitI32Const<ValueT>, Globals> for () {
    type Output = ValueT;
}

pub trait GlobalInitGet<Idx> {
    type Output;
}

impl<Globals, Idx, ValueT> GlobalInitGet<Idx> for Globals
where
    Globals: Get<Idx, Output = WasmGlobal<GlobalConst, WasmI32<ValueT>>>,
{
    type Output = ValueT;
}

impl<Idx, Globals> ResolveInitExprValue<InitGlobalGet<Idx>, Globals> for ()
where
    Globals: GlobalInitGet<Idx>,
{
    type Output = <Globals as GlobalInitGet<Idx>>::Output;
}

pub trait AppendDefinedGlobal<Existing, Mutability, InitExpr> {
    type Output;
}

impl<Existing, Mutability, InitExpr> AppendDefinedGlobal<Existing, Mutability, InitExpr> for ()
where
    (): ResolveInitExprValue<InitExpr, Existing>,
    Existing: Append<WasmGlobal<Mutability, WasmI32<<() as ResolveInitExprValue<InitExpr, Existing>>::Output>>>,
{
    type Output =
        <Existing as Append<WasmGlobal<Mutability, WasmI32<<() as ResolveInitExprValue<InitExpr, Existing>>::Output>>>>::Output;
}

pub trait AppendDefinedGlobals<Existing, Decls> {
    type Output;
}

impl<Existing> AppendDefinedGlobals<Existing, TTerm> for () {
    type Output = Existing;
}

impl<Existing, Mutability, InitExpr, Tail> AppendDefinedGlobals<Existing, TArr<WasmGlobalDecl<Mutability, InitExpr>, Tail>>
    for ()
where
    (): AppendDefinedGlobal<Existing, Mutability, InitExpr>,
    (): AppendDefinedGlobals<<() as AppendDefinedGlobal<Existing, Mutability, InitExpr>>::Output, Tail>,
{
    type Output =
        <() as AppendDefinedGlobals<<() as AppendDefinedGlobal<Existing, Mutability, InitExpr>>::Output, Tail>>::Output;
}

pub trait ApplyDataSegments<Memory, Segments, Globals> {
    type Output;
}

impl<Memory, Globals> ApplyDataSegments<Memory, TTerm, Globals> for () {
    type Output = Memory;
}

pub trait ResolveDataOffset<Expr, Globals> {
    type Output;
}

impl<Expr, Globals> ResolveDataOffset<Expr, Globals> for ()
where
    (): ResolveInitExprValue<Expr, Globals>,
{
    type Output = <() as ResolveInitExprValue<Expr, Globals>>::Output;
}

impl<Memory, OffsetExpr, Bytes, Tail, Globals> ApplyDataSegments<Memory, TArr<WasmDataSegment<OffsetExpr, Bytes>, Tail>, Globals>
    for ()
where
    (): ResolveDataOffset<OffsetExpr, Globals>,
    Memory: MemoryWriteBytes<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, Bytes>,
    (): ApplyDataSegments<
        <Memory as MemoryWriteBytes<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, Bytes>>::Output,
        Tail,
        Globals,
    >,
{
    type Output = <() as ApplyDataSegments<
        <Memory as MemoryWriteBytes<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, Bytes>>::Output,
        Tail,
        Globals,
    >>::Output;
}

pub trait InstantiateMemoryStore<MemoryDecl, Globals> {
    type Output;
}

impl<MinPages, MaxPages, Segments, Globals> InstantiateMemoryStore<WasmMemoryDecl<MinPages, MaxPages, Segments>, Globals>
    for ()
where
    MaxPages: ToStoreMax,
    (): ApplyDataSegments<WasmMemory<MinPages, <MaxPages as ToStoreMax>::Output, TTerm>, Segments, Globals>,
{
    type Output = <() as ApplyDataSegments<
        WasmMemory<MinPages, <MaxPages as ToStoreMax>::Output, TTerm>,
        Segments,
        Globals,
    >>::Output;
}

pub trait ApplyElemSegments<Table, Segments, Globals> {
    type Output;
}

impl<Table, Globals> ApplyElemSegments<Table, TTerm, Globals> for () {
    type Output = Table;
}

impl<Table, TableIdx, OffsetExpr, FuncIndices, Tail, Globals> ApplyElemSegments<Table, TArr<WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>, Tail>, Globals>
    for ()
where
    (): ResolveDataOffset<OffsetExpr, Globals>,
    Table: TableWriteRefs<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, FuncIndices>,
    (): ApplyElemSegments<
        <Table as TableWriteRefs<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, FuncIndices>>::Output,
        Tail,
        Globals,
    >,
{
    type Output = <() as ApplyElemSegments<
        <Table as TableWriteRefs<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, FuncIndices>>::Output,
        Tail,
        Globals,
    >>::Output;
}

pub trait AppendDefinedTables<Existing, Decls, Globals> {
    type Output;
}

impl<Existing, Globals> AppendDefinedTables<Existing, TTerm, Globals> for () {
    type Output = Existing;
}

impl<Existing, Min, Max, ElemSegments, Tail, Globals> AppendDefinedTables<Existing, TArr<WasmTableDecl<Min, Max, ElemSegments>, Tail>, Globals>
    for ()
where
    Max: ToStoreMax,
    (): ApplyElemSegments<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>, ElemSegments, Globals>,
    Existing: Append<<() as ApplyElemSegments<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>, ElemSegments, Globals>>::Output>,
    (): AppendDefinedTables<
        <Existing as Append<<() as ApplyElemSegments<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>, ElemSegments, Globals>>::Output>>::Output,
        Tail,
        Globals,
    >,
{
    type Output = <() as AppendDefinedTables<
        <Existing as Append<<() as ApplyElemSegments<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>, ElemSegments, Globals>>::Output>>::Output,
        Tail,
        Globals,
    >>::Output;
}

pub trait FuncSpaceTypes {
    type Output;
}

pub trait FuncSpaceFuncs {
    type Output;
}

impl<Types, Funcs> FuncSpaceTypes for WasmFuncSpace<Types, Funcs> {
    type Output = Types;
}

impl<Types, Funcs> FuncSpaceFuncs for WasmFuncSpace<Types, Funcs> {
    type Output = Funcs;
}

impl<FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start>
    Instantiate<()> for WasmModule<TTerm, FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start>
where
    FuncSpace: FuncSpaceTypes + FuncSpaceFuncs,
    (): AppendDefinedGlobals<TTerm, GlobalsDecl>,
    (): InstantiateMemoryStore<MemoryDecl, <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output>,
    (): AppendDefinedTables<TTerm, TablesDecl, <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output>,
{
    type Output = WasmInstance<
        WasmResolvedModule<
            <FuncSpace as FuncSpaceFuncs>::Output,
            <FuncSpace as FuncSpaceTypes>::Output,
            Exports,
        >,
        WasmStore<
            <() as InstantiateMemoryStore<MemoryDecl, <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output>>::Output,
            <() as AppendDefinedTables<TTerm, TablesDecl, <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output>>::Output,
            <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output,
        >,
        Start,
    >;
}

impl<FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start>
    Instantiate<WasmHostEnv<TTerm, TTerm, TTerm, TTerm>>
    for WasmModule<TTerm, FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start>
where
    WasmModule<TTerm, FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start>:
        Instantiate<()>,
{
    type Output =
        <WasmModule<TTerm, FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start> as Instantiate<()>>::Output;
}
