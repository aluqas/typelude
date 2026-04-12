use typelude_col::{TArr, TTerm};
use typelude_std::core::{Append, Concat, Get};

use crate::{
    helpers::{
        import::{
            ImportedFuncCompat, ImportedGlobalCompat, ImportedMemoryCompat, ImportedTableCompat,
            ResolveHostFuncBinding, ResolveHostGlobalBinding, ResolveHostMemoryBinding,
            ResolveHostTableBinding,
        },
        i32::U4294967295,
        memory::MemoryWriteBytes,
        table::TableWriteRefs,
    },
    module::{
        GlobalConst, ImportFunc, ImportGlobal, ImportMemory, ImportTable, InitGlobalGet,
        InitI32Const, NoLimit, WasmDataSegment, WasmElemSegment, WasmFuncSpace, WasmGlobalDecl,
        WasmHostEnv, WasmImport, WasmInstance, WasmMemoryDecl, WasmModule, WasmResolvedModule,
        WasmTableDecl,
    },
    state::{WasmGlobal, WasmMemory, WasmStore, WasmTable},
    value::WasmI32,
};

pub trait Instantiate<Env> {
    type Output;
}

#[doc(hidden)]
pub struct NoImportedMemory;

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

pub trait BuildImportedFuncs<Imports, Bindings> {
    type Output;
}

impl<Bindings> BuildImportedFuncs<TTerm, Bindings> for () {
    type Output = TTerm;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings> BuildImportedFuncs<TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>, Bindings>
    for ()
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
    BuildImportedFuncs<TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>, Bindings>
    for ()
where
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = <() as BuildImportedFuncs<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Bindings>
    BuildImportedFuncs<TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>, Bindings>
    for ()
where
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = <() as BuildImportedFuncs<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Min, Max, Tail, Bindings>
    BuildImportedFuncs<TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>, Bindings>
    for ()
where
    (): BuildImportedFuncs<Tail, Bindings>,
{
    type Output = <() as BuildImportedFuncs<Tail, Bindings>>::Output;
}

pub trait BuildImportedGlobals<Imports, Bindings> {
    type Output;
}

impl<Bindings> BuildImportedGlobals<TTerm, Bindings> for () {
    type Output = TTerm;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings>
    BuildImportedGlobals<TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>, Bindings>
    for ()
where
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = <() as BuildImportedGlobals<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Bindings>
    BuildImportedGlobals<TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>, Bindings>
    for ()
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
    BuildImportedGlobals<TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>, Bindings>
    for ()
where
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = <() as BuildImportedGlobals<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Min, Max, Tail, Bindings>
    BuildImportedGlobals<TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>, Bindings>
    for ()
where
    (): BuildImportedGlobals<Tail, Bindings>,
{
    type Output = <() as BuildImportedGlobals<Tail, Bindings>>::Output;
}

pub trait ResolveImportedMemory<Imports, Bindings> {
    type Output;
}

impl<Bindings> ResolveImportedMemory<TTerm, Bindings> for () {
    type Output = NoImportedMemory;
}

impl<ModuleName, FieldName, FuncType, Tail, Bindings>
    ResolveImportedMemory<TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>, Bindings>
    for ()
where
    (): ResolveImportedMemory<Tail, Bindings>,
{
    type Output = <() as ResolveImportedMemory<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Bindings>
    ResolveImportedMemory<TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>, Bindings>
    for ()
where
    (): ResolveImportedMemory<Tail, Bindings>,
{
    type Output = <() as ResolveImportedMemory<Tail, Bindings>>::Output;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Bindings>
    ResolveImportedMemory<TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>, Bindings>
    for ()
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
    ResolveImportedMemory<TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, Tail>, Bindings>
    for ()
where
    (): ResolveImportedMemory<Tail, Bindings>,
{
    type Output = <() as ResolveImportedMemory<Tail, Bindings>>::Output;
}

pub trait MaterializeMemoryStore<ImportedMemory, MemoryDecl, Globals> {
    type Output;
}

impl<MemoryDecl, Globals> MaterializeMemoryStore<NoImportedMemory, MemoryDecl, Globals> for ()
where
    (): InstantiateMemoryStore<MemoryDecl, Globals>,
{
    type Output = <() as InstantiateMemoryStore<MemoryDecl, Globals>>::Output;
}

impl<Pages, ActualMax, Cells, MinPages, MaxPages, Segments, Globals>
    MaterializeMemoryStore<WasmMemory<Pages, ActualMax, Cells>, WasmMemoryDecl<MinPages, MaxPages, Segments>, Globals>
    for ()
where
    (): ApplyDataSegments<WasmMemory<Pages, ActualMax, Cells>, Segments, Globals>,
{
    type Output =
        <() as ApplyDataSegments<WasmMemory<Pages, ActualMax, Cells>, Segments, Globals>>::Output;
}

pub trait InstantiateImportedTables<Imports, Decls, Bindings, Globals> {
    type Imported;
    type RemainingDecls;
}

impl<Decls, Bindings, Globals> InstantiateImportedTables<TTerm, Decls, Bindings, Globals> for () {
    type Imported = TTerm;
    type RemainingDecls = Decls;
}

impl<ModuleName, FieldName, FuncType, Tail, Decls, Bindings, Globals>
    InstantiateImportedTables<TArr<WasmImport<ModuleName, FieldName, ImportFunc<FuncType>>, Tail>, Decls, Bindings, Globals>
    for ()
where
    (): InstantiateImportedTables<Tail, Decls, Bindings, Globals>,
{
    type Imported = <() as InstantiateImportedTables<Tail, Decls, Bindings, Globals>>::Imported;
    type RemainingDecls =
        <() as InstantiateImportedTables<Tail, Decls, Bindings, Globals>>::RemainingDecls;
}

impl<ModuleName, FieldName, Mutability, ValueType, Tail, Decls, Bindings, Globals>
    InstantiateImportedTables<TArr<WasmImport<ModuleName, FieldName, ImportGlobal<Mutability, ValueType>>, Tail>, Decls, Bindings, Globals>
    for ()
where
    (): InstantiateImportedTables<Tail, Decls, Bindings, Globals>,
{
    type Imported = <() as InstantiateImportedTables<Tail, Decls, Bindings, Globals>>::Imported;
    type RemainingDecls =
        <() as InstantiateImportedTables<Tail, Decls, Bindings, Globals>>::RemainingDecls;
}

impl<ModuleName, FieldName, MinPages, MaxPages, Tail, Decls, Bindings, Globals>
    InstantiateImportedTables<TArr<WasmImport<ModuleName, FieldName, ImportMemory<MinPages, MaxPages>>, Tail>, Decls, Bindings, Globals>
    for ()
where
    (): InstantiateImportedTables<Tail, Decls, Bindings, Globals>,
{
    type Imported = <() as InstantiateImportedTables<Tail, Decls, Bindings, Globals>>::Imported;
    type RemainingDecls =
        <() as InstantiateImportedTables<Tail, Decls, Bindings, Globals>>::RemainingDecls;
}

impl<ModuleName, FieldName, Min, Max, TailImports, ElemSegments, TailDecls, Bindings, Globals>
    InstantiateImportedTables<
        TArr<WasmImport<ModuleName, FieldName, ImportTable<Min, Max>>, TailImports>,
        TArr<WasmTableDecl<Min, Max, ElemSegments>, TailDecls>,
        Bindings,
        Globals,
    > for ()
where
    Bindings: ResolveHostTableBinding<ModuleName, FieldName>,
    <Bindings as ResolveHostTableBinding<ModuleName, FieldName>>::Output:
        ImportedTableCompat<ImportTable<Min, Max>>,
    (): ApplyElemSegments<
        <<Bindings as ResolveHostTableBinding<ModuleName, FieldName>>::Output as ImportedTableCompat<
            ImportTable<Min, Max>,
        >>::Output,
        ElemSegments,
        Globals,
    >,
    (): InstantiateImportedTables<TailImports, TailDecls, Bindings, Globals>,
{
    type Imported = TArr<
        <() as ApplyElemSegments<
            <<Bindings as ResolveHostTableBinding<ModuleName, FieldName>>::Output as ImportedTableCompat<
                ImportTable<Min, Max>,
            >>::Output,
            ElemSegments,
            Globals,
        >>::Output,
        <() as InstantiateImportedTables<TailImports, TailDecls, Bindings, Globals>>::Imported,
    >;
    type RemainingDecls =
        <() as InstantiateImportedTables<TailImports, TailDecls, Bindings, Globals>>::RemainingDecls;
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

impl<
        Imports,
        FuncSpace,
        MemoryDecl,
        TablesDecl,
        GlobalsDecl,
        Exports,
        Start,
        FuncBindings,
        GlobalBindings,
        MemoryBindings,
        TableBindings,
    > Instantiate<WasmHostEnv<FuncBindings, GlobalBindings, MemoryBindings, TableBindings>>
    for WasmModule<Imports, FuncSpace, MemoryDecl, TablesDecl, GlobalsDecl, Exports, Start>
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
        MemoryDecl,
        <() as AppendDefinedGlobals<
            <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
            GlobalsDecl,
        >>::Output,
    >,
    (): InstantiateImportedTables<
        Imports,
        TablesDecl,
        TableBindings,
        <() as AppendDefinedGlobals<
            <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
            GlobalsDecl,
        >>::Output,
    >,
    (): AppendDefinedTables<
        <() as InstantiateImportedTables<
            Imports,
            TablesDecl,
            TableBindings,
            <() as AppendDefinedGlobals<
                <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                GlobalsDecl,
            >>::Output,
        >>::Imported,
        <() as InstantiateImportedTables<
            Imports,
            TablesDecl,
            TableBindings,
            <() as AppendDefinedGlobals<
                <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                GlobalsDecl,
            >>::Output,
        >>::RemainingDecls,
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
                MemoryDecl,
                <() as AppendDefinedGlobals<
                    <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                    GlobalsDecl,
                >>::Output,
            >>::Output,
            <() as AppendDefinedTables<
                <() as InstantiateImportedTables<
                    Imports,
                    TablesDecl,
                    TableBindings,
                    <() as AppendDefinedGlobals<
                        <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                        GlobalsDecl,
                    >>::Output,
                >>::Imported,
                <() as InstantiateImportedTables<
                    Imports,
                    TablesDecl,
                    TableBindings,
                    <() as AppendDefinedGlobals<
                        <() as BuildImportedGlobals<Imports, GlobalBindings>>::Output,
                        GlobalsDecl,
                    >>::Output,
                >>::RemainingDecls,
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
