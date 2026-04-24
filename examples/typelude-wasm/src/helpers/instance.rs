//! モジュールインスタンス化の helper 群。
//!
//! このファイルは `WasmModule<...>` を `WasmInstance<...>` に変換するための
//! 型レベル pass をまとめています。処理順は概ね import 関数の構築、import
//! global の構築、 定義済み global の初期化、memory の materialize と data
//! segment 適用、 table の materialize と elem segment 適用、resolved module
//! の組み立てです。
//!
//! 失敗の多くは trait 未解決として表面化します。例えば host binding
//! がない場合は `MissingHost*Binding` 型に到達し、その後の compat trait
//! を満たせず compile-time に失敗します。

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Append, Concat, Get, Set};

use crate::{
    helpers::{
        i32::U4294967295,
        import::{
            ImportedFuncCompat, ImportedGlobalCompat, ImportedMemoryCompat, ImportedTableCompat,
            ResolveHostFuncBinding, ResolveHostGlobalBinding, ResolveHostMemoryBinding,
            ResolveHostTableBinding,
        },
        memory::MemoryWriteBytes,
        table::TableWriteRefs,
    },
    module::{
        GlobalConst, ImportFunc, ImportGlobal, ImportMemory, ImportTable, InitGlobalGet,
        InitI32Const, InitI64Const, NoLimit, NoMemoryDecl, WasmConstExpr, WasmDataSegment,
        WasmElemSegment, WasmFuncSpace, WasmGlobalDecl, WasmHostEnv, WasmImport, WasmInstance,
        WasmMemoryDecl, WasmModule, WasmModuleMemory, WasmModuleTables, WasmResolvedModule,
        WasmTableDecl,
    },
    opcode::{OpGlobalGet, OpI32Const, OpI64Const},
    state::{WasmGlobal, WasmMemory, WasmStore, WasmTable},
    value::{WasmI32, WasmI64},
};

/// `WasmModule` を `WasmInstance` へインスタンス化する helper family。
///
/// `Env` は `WasmHostEnv` です。出力は `WasmInstance<WasmResolvedModule<...>,
/// WasmStore<...>, Start>` になり、`run.rs` の build helper がこの instance
/// から初期 `WasmState` を作ります。
pub trait Instantiate<Env> {
    type Output;
}

#[doc(hidden)]
/// import された memory が存在しないことを表す内部マーカー。
pub struct NoImportedMemory;

/// module 宣言上の max pages を store 表現へ正規化する helper。
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

/// const expr を runtime 値へ解決する helper family。
pub trait ResolveInitExprRuntime<Expr, Globals> {
    type Output;
}

impl<ValueT, Globals>
    ResolveInitExprRuntime<WasmConstExpr<TArr<OpI32Const<ValueT>, TTerm>>, Globals> for ()
{
    type Output = WasmI32<ValueT>;
}

impl<ValueT, Globals>
    ResolveInitExprRuntime<WasmConstExpr<TArr<OpI64Const<ValueT>, TTerm>>, Globals> for ()
{
    type Output = WasmI64<ValueT>;
}

impl<ValueT, Globals> ResolveInitExprRuntime<InitI32Const<ValueT>, Globals> for () {
    type Output = WasmI32<ValueT>;
}

impl<ValueT, Globals> ResolveInitExprRuntime<InitI64Const<ValueT>, Globals> for () {
    type Output = WasmI64<ValueT>;
}

/// immutable global を初期化式中で読み出す helper。
pub trait GlobalInitGet<Idx> {
    type Output;
}

impl<Globals, Idx, ValueT> GlobalInitGet<Idx> for Globals
where
    Globals: Get<Idx, Output = WasmGlobal<GlobalConst, ValueT>>,
{
    type Output = ValueT;
}

impl<Idx, Globals> ResolveInitExprRuntime<InitGlobalGet<Idx>, Globals> for ()
where
    Globals: GlobalInitGet<Idx>,
{
    type Output = <Globals as GlobalInitGet<Idx>>::Output;
}

impl<Idx, Globals> ResolveInitExprRuntime<WasmConstExpr<TArr<OpGlobalGet<Idx>, TTerm>>, Globals>
    for ()
where
    Globals: GlobalInitGet<Idx>,
{
    type Output = <Globals as GlobalInitGet<Idx>>::Output;
}

/// 初期化式結果から offset 整数値を取り出す helper。
pub trait InitExprOffsetValue {
    type Output;
}

impl<ValueT> InitExprOffsetValue for WasmI32<ValueT> {
    type Output = ValueT;
}

/// const expr を offset 値として解決する helper。
pub trait ResolveInitExprValue<Expr, Globals> {
    type Output;
}

impl<Expr, Globals> ResolveInitExprValue<Expr, Globals> for ()
where
    (): ResolveInitExprRuntime<Expr, Globals>,
    <() as ResolveInitExprRuntime<Expr, Globals>>::Output: InitExprOffsetValue,
{
    type Output =
        <<() as ResolveInitExprRuntime<Expr, Globals>>::Output as InitExprOffsetValue>::Output;
}

/// 定義済み global 宣言を既存 global 列へ 1 つ追加する helper。
pub trait AppendDefinedGlobal<Existing, Mutability, InitExpr> {
    type Output;
}

impl<Existing, Mutability, InitExpr> AppendDefinedGlobal<Existing, Mutability, InitExpr> for ()
where
    (): ResolveInitExprRuntime<InitExpr, Existing>,
    Existing:
        Append<WasmGlobal<Mutability, <() as ResolveInitExprRuntime<InitExpr, Existing>>::Output>>,
{
    type Output = <Existing as Append<
        WasmGlobal<Mutability, <() as ResolveInitExprRuntime<InitExpr, Existing>>::Output>,
    >>::Output;
}

/// global 宣言列を既存 global 列へ畳み込む helper。
pub trait AppendDefinedGlobals<Existing, Decls> {
    type Output;
}

impl<Existing> AppendDefinedGlobals<Existing, TTerm> for () {
    type Output = Existing;
}

impl<Existing, Mutability, InitExpr, Tail>
    AppendDefinedGlobals<Existing, TArr<WasmGlobalDecl<Mutability, InitExpr>, Tail>> for ()
where
    (): AppendDefinedGlobal<Existing, Mutability, InitExpr>,
    (): AppendDefinedGlobals<
            <() as AppendDefinedGlobal<Existing, Mutability, InitExpr>>::Output,
            Tail,
        >,
{
    type Output = <() as AppendDefinedGlobals<
        <() as AppendDefinedGlobal<Existing, Mutability, InitExpr>>::Output,
        Tail,
    >>::Output;
}

/// data segment 群を memory へ適用する helper。
pub trait ApplyDataSegments<Memory, Segments, Globals> {
    type Output;
}

impl<Memory, Globals> ApplyDataSegments<Memory, TTerm, Globals> for () {
    type Output = Memory;
}

/// data / elem segment の offset 式を解決する helper。
pub trait ResolveDataOffset<Expr, Globals> {
    type Output;
}

impl<Expr, Globals> ResolveDataOffset<Expr, Globals> for ()
where
    (): ResolveInitExprValue<Expr, Globals>,
{
    type Output = <() as ResolveInitExprValue<Expr, Globals>>::Output;
}

impl<Memory, OffsetExpr, Bytes, Tail, Globals>
    ApplyDataSegments<Memory, TArr<WasmDataSegment<OffsetExpr, Bytes>, Tail>, Globals> for ()
where
    (): ResolveDataOffset<OffsetExpr, Globals>,
    Memory: MemoryWriteBytes<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, Bytes>,
    (): ApplyDataSegments<
            <Memory as MemoryWriteBytes<
                <() as ResolveDataOffset<OffsetExpr, Globals>>::Output,
                Bytes,
            >>::Output,
            Tail,
            Globals,
        >,
{
    type Output = <() as ApplyDataSegments<
        <Memory as MemoryWriteBytes<
            <() as ResolveDataOffset<OffsetExpr, Globals>>::Output,
            Bytes,
        >>::Output,
        Tail,
        Globals,
    >>::Output;
}

/// module memory 宣言から store 上の memory 実体を構築する helper。
pub trait InstantiateMemoryStore<MemoryDecl, Globals> {
    type Output;
}

impl<Globals> InstantiateMemoryStore<NoMemoryDecl, Globals> for () {
    type Output = WasmMemory<typenum::U0, U4294967295, TTerm>;
}

impl<MinPages, MaxPages, Globals>
    InstantiateMemoryStore<WasmMemoryDecl<MinPages, MaxPages>, Globals> for ()
where
    MaxPages: ToStoreMax,
{
    type Output = WasmMemory<MinPages, <MaxPages as ToStoreMax>::Output, TTerm>;
}

/// elem segment 群を table へ適用する helper。
pub trait ApplyElemSegments<Table, Segments, Globals> {
    type Output;
}

impl<Table, Globals> ApplyElemSegments<Table, TTerm, Globals> for () {
    type Output = Table;
}

impl<Table, TableIdx, OffsetExpr, FuncIndices, Tail, Globals>
    ApplyElemSegments<
        Table,
        TArr<WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>, Tail>,
        Globals,
    > for ()
where
    (): ResolveDataOffset<OffsetExpr, Globals>,
    Table: TableWriteRefs<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, FuncIndices>,
    (): ApplyElemSegments<
            <Table as TableWriteRefs<
                <() as ResolveDataOffset<OffsetExpr, Globals>>::Output,
                FuncIndices,
            >>::Output,
            Tail,
            Globals,
        >,
{
    type Output = <() as ApplyElemSegments<
        <Table as TableWriteRefs<
            <() as ResolveDataOffset<OffsetExpr, Globals>>::Output,
            FuncIndices,
        >>::Output,
        Tail,
        Globals,
    >>::Output;
}

/// table 宣言列を既存 table 列へ追加する helper。
pub trait AppendDefinedTables<Existing, Decls> {
    type Output;
}

impl<Existing> AppendDefinedTables<Existing, TTerm> for () {
    type Output = Existing;
}

impl<Existing, Min, Max, Tail> AppendDefinedTables<Existing, TArr<WasmTableDecl<Min, Max>, Tail>>
    for ()
where
    Max: ToStoreMax,
    Existing: Append<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>>,
    (): AppendDefinedTables<
            <Existing as Append<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>>>::Output,
            Tail,
        >,
{
    type Output = <() as AppendDefinedTables<
        <Existing as Append<WasmTable<Min, <Max as ToStoreMax>::Output, TTerm>>>::Output,
        Tail,
    >>::Output;
}

/// function space から型一覧を取り出す helper。
pub trait FuncSpaceTypes {
    type Output;
}

/// function space から関数一覧を取り出す helper。
pub trait FuncSpaceFuncs {
    type Output;
}

impl<Types, Funcs> FuncSpaceTypes for WasmFuncSpace<Types, Funcs> {
    type Output = Types;
}

impl<Types, Funcs> FuncSpaceFuncs for WasmFuncSpace<Types, Funcs> {
    type Output = Funcs;
}

/// import 宣言列と host bindings から import 関数列を構築する helper。
pub trait BuildImportedFuncs<Imports, Bindings> {
    type Output;
}

impl<Bindings> BuildImportedFuncs<TTerm, Bindings> for () {
    type Output = TTerm;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings>
    BuildImportedFuncs<
        TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>,
        Bindings,
    > for ()
where
    Bindings: ResolveHostFuncBinding<ModuleName, FieldName>,
    <Bindings as ResolveHostFuncBinding<ModuleName, FieldName>>::Output:
        ImportedFuncCompat<ImportFunc<FuncType>>,
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = TArr<
        <<Bindings as ResolveHostFuncBinding<ModuleName, FieldName>>::Output as ImportedFuncCompat<
            ImportFunc<FuncType>,
        >>::Output,
        <() as BuildImportedFuncs<Tail, Bindings>>::Output,
    >;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Bindings>
    BuildImportedFuncs<
        TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = <() as BuildImportedFuncs<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Bindings>
    BuildImportedFuncs<
        TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = <() as BuildImportedFuncs<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Min, Max, Tail, Bindings>
    BuildImportedFuncs<
        TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = <() as BuildImportedFuncs<Tail, Bindings>>::Output;
}

/// import 宣言列と host bindings から import global 列を構築する helper。
pub trait BuildImportedGlobals<Imports, Bindings> {
    type Output;
}

impl<Bindings> BuildImportedGlobals<TTerm, Bindings> for () {
    type Output = TTerm;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings>
    BuildImportedGlobals<
        TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = <() as BuildImportedGlobals<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Bindings>
    BuildImportedGlobals<
        TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>,
        Bindings,
    > for ()
where
    Bindings: ResolveHostGlobalBinding<ModuleName, FieldName>,
    <Bindings as ResolveHostGlobalBinding<ModuleName, FieldName>>::Output:
        ImportedGlobalCompat<ImportGlobal<Mutability, ValueType>>,
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = TArr<
        <<Bindings as ResolveHostGlobalBinding<ModuleName, FieldName>>::Output as ImportedGlobalCompat<
            ImportGlobal<Mutability, ValueType>,
        >>::Output,
        <() as BuildImportedGlobals<Tail, Bindings>>::Output,
    >;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Bindings>
    BuildImportedGlobals<
        TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = <() as BuildImportedGlobals<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Min, Max, Tail, Bindings>
    BuildImportedGlobals<
        TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = <() as BuildImportedGlobals<Tail, Bindings>>::Output;
}

/// import 宣言列と host bindings から import memory を解決する helper。
pub trait ResolveImportedMemory<Imports, Bindings> {
    type Output;
}

impl<Bindings> ResolveImportedMemory<TTerm, Bindings> for () {
    type Output = NoImportedMemory;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings>
    ResolveImportedMemory<
        TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>,
        Bindings,
    > for ()
where
    (): ResolveImportedMemory<Tail, Bindings>,
{
    type Output = <() as ResolveImportedMemory<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Bindings>
    ResolveImportedMemory<
        TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>,
        Bindings,
    > for ()
where
    (): ResolveImportedMemory<Tail, Bindings>,
{
    type Output = <() as ResolveImportedMemory<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Bindings>
    ResolveImportedMemory<
        TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>,
        Bindings,
    > for ()
where
    Bindings: ResolveHostMemoryBinding<ModuleName, FieldName>,
    <Bindings as ResolveHostMemoryBinding<ModuleName, FieldName>>::Output:
        ImportedMemoryCompat<ImportMemory<MinPages, MaxPages>>,
{
    type Output =
        <<Bindings as ResolveHostMemoryBinding<ModuleName, FieldName>>::Output as ImportedMemoryCompat<
            ImportMemory<MinPages, MaxPages>,
        >>::Output;
}

impl<ModuleName, FieldName, Min, Max, Tail, Bindings>
    ResolveImportedMemory<
        TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>,
        Bindings,
    > for ()
where
    (): ResolveImportedMemory<Tail, Bindings>,
{
    type Output = <() as ResolveImportedMemory<Tail, Bindings>>::Output;
}

/// import memory と memory section から store memory を materialize する
/// helper。
///
/// memory import がある場合は import された memory を使い、ない場合は module
/// 側の `WasmMemoryDecl` から空 memory を作ります。その後、data segment
/// を順に書き込みます。
pub trait MaterializeMemoryStore<ImportedMemory, MemorySection, Globals> {
    type Output;
}

impl<MemoryDecl, DataSegments, Globals>
    MaterializeMemoryStore<NoImportedMemory, WasmModuleMemory<MemoryDecl, DataSegments>, Globals>
    for ()
where
    (): InstantiateMemoryStore<MemoryDecl, Globals>,
    (): ApplyDataSegments<
            <() as InstantiateMemoryStore<MemoryDecl, Globals>>::Output,
            DataSegments,
            Globals,
        >,
{
    type Output = <() as ApplyDataSegments<
        <() as InstantiateMemoryStore<MemoryDecl, Globals>>::Output,
        DataSegments,
        Globals,
    >>::Output;
}

impl<Pages, ActualMax, Cells, DataSegments, Globals>
    MaterializeMemoryStore<
        WasmMemory<Pages, ActualMax, Cells>,
        WasmModuleMemory<NoMemoryDecl, DataSegments>,
        Globals,
    > for ()
where
    (): ApplyDataSegments<WasmMemory<Pages, ActualMax, Cells>, DataSegments, Globals>,
{
    type Output = <() as ApplyDataSegments<
        WasmMemory<Pages, ActualMax, Cells>,
        DataSegments,
        Globals,
    >>::Output;
}

/// import 宣言列と host bindings から import table 列を構築する helper。
pub trait BuildImportedTables<Imports, Bindings> {
    type Output;
}

impl<Bindings> BuildImportedTables<TTerm, Bindings> for () {
    type Output = TTerm;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings>
    BuildImportedTables<
        TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedTables<Tail, Bindings>,
{
    type Output = <() as BuildImportedTables<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Bindings>
    BuildImportedTables<
        TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedTables<Tail, Bindings>,
{
    type Output = <() as BuildImportedTables<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Bindings>
    BuildImportedTables<
        TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>,
        Bindings,
    > for ()
where
    (): BuildImportedTables<Tail, Bindings>,
{
    type Output = <() as BuildImportedTables<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Min, Max, Tail, Bindings>
    BuildImportedTables<
        TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>,
        Bindings,
    > for ()
where
    Bindings: ResolveHostTableBinding<ModuleName, FieldName>,
    <Bindings as ResolveHostTableBinding<ModuleName, FieldName>>::Output:
        ImportedTableCompat<ImportTable<Min, Max>>,
    (): BuildImportedTables<Tail, Bindings>,
{
    type Output = TArr<
        <<Bindings as ResolveHostTableBinding<ModuleName, FieldName>>::Output as ImportedTableCompat<
            ImportTable<Min, Max>,
        >>::Output,
        <() as BuildImportedTables<Tail, Bindings>>::Output,
    >;
}

/// 1 つの elem segment を table 群へ書き込む helper。
pub trait WriteElemSegment<Tables, Segment, Globals> {
    type Output;
}

impl<Tables, TableIdx, OffsetExpr, FuncIndices, Globals>
    WriteElemSegment<Tables, WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>, Globals> for ()
where
    (): ResolveDataOffset<OffsetExpr, Globals>,
    Tables: Get<TableIdx>,
    <Tables as Get<TableIdx>>::Output:
        TableWriteRefs<<() as ResolveDataOffset<OffsetExpr, Globals>>::Output, FuncIndices>,
    Tables: Set<
            TableIdx,
            <<Tables as Get<TableIdx>>::Output as TableWriteRefs<
                <() as ResolveDataOffset<OffsetExpr, Globals>>::Output,
                FuncIndices,
            >>::Output,
        >,
{
    type Output = <Tables as Set<
        TableIdx,
        <<Tables as Get<TableIdx>>::Output as TableWriteRefs<
            <() as ResolveDataOffset<OffsetExpr, Globals>>::Output,
            FuncIndices,
        >>::Output,
    >>::Output;
}

/// elem segment 群を table 群へ適用する helper。
pub trait ApplyElemSegmentsToTables<Tables, Segments, Globals> {
    type Output;
}

impl<Tables, Globals> ApplyElemSegmentsToTables<Tables, TTerm, Globals> for () {
    type Output = Tables;
}

impl<Tables, Segment, Tail, Globals>
    ApplyElemSegmentsToTables<Tables, TArr<Segment, Tail>, Globals> for ()
where
    (): WriteElemSegment<Tables, Segment, Globals>,
    (): ApplyElemSegmentsToTables<
            <() as WriteElemSegment<Tables, Segment, Globals>>::Output,
            Tail,
            Globals,
        >,
{
    type Output = <() as ApplyElemSegmentsToTables<
        <() as WriteElemSegment<Tables, Segment, Globals>>::Output,
        Tail,
        Globals,
    >>::Output;
}

/// import tables と table section から最終的な table 群を materialize する
/// helper。
///
/// import table と定義済み table を連結した後、elem segment を適用します。
/// `call_indirect` はここで書き込まれた `TableEntry` を runtime lookup します。
pub trait MaterializeTables<Imports, TablesSection, Bindings, Globals> {
    type Output;
}

impl<Imports, TableDecls, ElemSegments, Bindings, Globals>
    MaterializeTables<Imports, WasmModuleTables<TableDecls, ElemSegments>, Bindings, Globals>
    for ()
where
    (): BuildImportedTables<Imports, Bindings>,
    (): AppendDefinedTables<<() as BuildImportedTables<Imports, Bindings>>::Output, TableDecls>,
    (): ApplyElemSegmentsToTables<
            <() as AppendDefinedTables<
                <() as BuildImportedTables<Imports, Bindings>>::Output,
                TableDecls,
            >>::Output,
            ElemSegments,
            Globals,
        >,
{
    type Output = <() as ApplyElemSegmentsToTables<
        <() as AppendDefinedTables<
            <() as BuildImportedTables<Imports, Bindings>>::Output,
            TableDecls,
        >>::Output,
        ElemSegments,
        Globals,
    >>::Output;
}

impl<FuncSpace, MemorySection, TablesSection, GlobalsDecl, Exports, Start> Instantiate<()>
    for WasmModule<TTerm, FuncSpace, MemorySection, TablesSection, GlobalsDecl, Exports, Start>
where
    FuncSpace: FuncSpaceTypes + FuncSpaceFuncs,
    (): AppendDefinedGlobals<TTerm, GlobalsDecl>,
    (): MaterializeMemoryStore<
            NoImportedMemory,
            MemorySection,
            <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output,
        >,
    (): MaterializeTables<
            TTerm,
            TablesSection,
            TTerm,
            <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output,
        >,
{
    type Output = WasmInstance<
        WasmResolvedModule<
            <FuncSpace as FuncSpaceFuncs>::Output,
            <FuncSpace as FuncSpaceTypes>::Output,
            Exports,
        >,
        WasmStore<
            <() as MaterializeMemoryStore<
                NoImportedMemory,
                MemorySection,
                <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output,
            >>::Output,
            <() as MaterializeTables<
                TTerm,
                TablesSection,
                TTerm,
                <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output,
            >>::Output,
            <() as AppendDefinedGlobals<TTerm, GlobalsDecl>>::Output,
        >,
        Start,
    >;
}

impl<
    Imports,
    FuncSpace,
    MemorySection,
    TablesSection,
    GlobalsDecl,
    Exports,
    Start,
    FuncBindings,
    GlobalBindings,
    MemoryBindings,
    TableBindings,
> Instantiate<WasmHostEnv<FuncBindings, GlobalBindings, MemoryBindings, TableBindings>>
    for WasmModule<Imports, FuncSpace, MemorySection, TablesSection, GlobalsDecl, Exports, Start>
where
    FuncSpace: FuncSpaceTypes + FuncSpaceFuncs,
    (): BuildImportedFuncs<Imports, FuncBindings>,
    <() as BuildImportedFuncs<Imports, FuncBindings>>::Output:
        Concat<<FuncSpace as FuncSpaceFuncs>::Output>,
    (): BuildImportedGlobals<Imports, GlobalBindings>,
    (): AppendDefinedGlobals<
            <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
            GlobalsDecl,
        >,
    (): ResolveImportedMemory<Imports, MemoryBindings>,
    (): MaterializeMemoryStore<
            <() as ResolveImportedMemory<Imports, MemoryBindings>>::Output,
            MemorySection,
            <() as AppendDefinedGlobals<
                <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                GlobalsDecl,
            >>::Output,
        >,
    (): MaterializeTables<
            Imports,
            TablesSection,
            TableBindings,
            <() as AppendDefinedGlobals<
                <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                GlobalsDecl,
            >>::Output,
        >,
{
    type Output = WasmInstance<
        WasmResolvedModule<
            <<() as BuildImportedFuncs<Imports, FuncBindings>>::Output as Concat<
                <FuncSpace as FuncSpaceFuncs>::Output,
            >>::Output,
            <FuncSpace as FuncSpaceTypes>::Output,
            Exports,
        >,
        WasmStore<
            <() as MaterializeMemoryStore<
                <() as ResolveImportedMemory<Imports, MemoryBindings>>::Output,
                MemorySection,
                <() as AppendDefinedGlobals<
                    <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                    GlobalsDecl,
                >>::Output,
            >>::Output,
            <() as MaterializeTables<
                Imports,
                TablesSection,
                TableBindings,
                <() as AppendDefinedGlobals<
                    <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                    GlobalsDecl,
                >>::Output,
            >>::Output,
            <() as AppendDefinedGlobals<
                <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                GlobalsDecl,
            >>::Output,
        >,
        Start,
    >;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    type ImportAddMain = Fn2<TTerm, tarr![OpLocalGet<U0>, OpLocalGet<U1>, OpCall<U0>, OpReturn]>;
    type ImportAddModule = WasmModule<
        tarr![WasmImport<
            typelude_str::tstr!("host"),
            typelude_str::tstr!("add"),
            ImportFunc<WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>>,
        >],
        WasmFuncSpace<
            tarr![WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>],
            tarr![ImportAddMain],
        >,
        WasmModuleMemory<NoMemoryDecl, TTerm>,
        WasmModuleTables<TTerm, TTerm>,
        TTerm,
        tarr![WasmExport<typelude_str::tstr!("main"), ExportFunc<U1>>],
        NoStart,
    >;

    struct HostAdd;

    impl
        HostCall<
            WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>,
            Store<ZeroPages>,
            tarr![WasmI32<U2>, tarr![WasmI32<U3>]],
        > for HostAdd
    {
        type Output = HostCallResult<Store<ZeroPages>, tarr![WasmI32<U5>]>;
    }

    type ImportAddEnv = WasmHostEnv<
        tarr![HostFuncBinding<typelude_str::tstr!("host"), typelude_str::tstr!("add"), HostAdd>],
        TTerm,
        TTerm,
        TTerm,
    >;

    #[test]
    fn imported_function_calls_resolve_through_host_env() {
        type Final = InvokeFuncWithEnv<
            ImportAddModule,
            ImportAddEnv,
            U1,
            tarr![WasmI32<U2>, tarr![WasmI32<U3>]],
        >;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U5>]);
    }
}
