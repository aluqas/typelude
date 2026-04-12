use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};

use crate::{
    helpers::name::{BoolOutput, NameEq},
    module::{
        ExportFunc, ExportGlobal, ExportMemory, ExportTable, WasmExport, WasmInstance,
        WasmResolvedModule,
    },
};

pub struct MissingExport<Name>(pub PhantomData<Name>);

pub trait ResolveExportKind<Name> {
    type Output;
}

pub trait ResolveExportKindHelper<Name, Kind, Tail> {
    type Output;
}

impl<Name, Kind, Tail> ResolveExportKindHelper<Name, Kind, Tail> for typenum::B1 {
    type Output = Kind;
}

impl<Name, Kind, Tail> ResolveExportKindHelper<Name, Kind, Tail> for typenum::B0
where
    Tail: ResolveExportKind<Name>,
{
    type Output = <Tail as ResolveExportKind<Name>>::Output;
}

impl<Name, ExportName, Kind, Tail> ResolveExportKind<Name>
    for TArr<WasmExport<ExportName, Kind>, Tail>
where
    ExportName: NameEq<Name>,
    <ExportName as NameEq<Name>>::Output: BoolOutput,
    <<ExportName as NameEq<Name>>::Output as BoolOutput>::Output:
        ResolveExportKindHelper<Name, Kind, Tail>,
{
    type Output = <<<ExportName as NameEq<Name>>::Output as BoolOutput>::Output as ResolveExportKindHelper<
        Name,
        Kind,
        Tail,
    >>::Output;
}

impl<Name> ResolveExportKind<Name> for TTerm {
    type Output = MissingExport<Name>;
}

pub trait ExportKindToFunc {
    type Output;
}

impl<FuncIdx> ExportKindToFunc for ExportFunc<FuncIdx> {
    type Output = FuncIdx;
}

pub trait ExportKindToGlobal {
    type Output;
}

impl<GlobalIdx> ExportKindToGlobal for ExportGlobal<GlobalIdx> {
    type Output = GlobalIdx;
}

pub trait ExportKindToTable {
    type Output;
}

impl<TableIdx> ExportKindToTable for ExportTable<TableIdx> {
    type Output = TableIdx;
}

pub trait ExportKindToMemory {}

impl ExportKindToMemory for ExportMemory {}

pub trait ResolveExportFunc<Name> {
    type Output;
}

impl<Funcs, Types, Exports, Name> ResolveExportFunc<Name>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    Exports: ResolveExportKind<Name>,
    <Exports as ResolveExportKind<Name>>::Output: ExportKindToFunc,
{
    type Output = <<Exports as ResolveExportKind<Name>>::Output as ExportKindToFunc>::Output;
}

impl<Module, Store, Start, Name> ResolveExportFunc<Name> for WasmInstance<Module, Store, Start>
where
    Module: ResolveExportFunc<Name>,
{
    type Output = <Module as ResolveExportFunc<Name>>::Output;
}

pub trait ResolveExportGlobal<Name> {
    type Output;
}

impl<Funcs, Types, Exports, Name> ResolveExportGlobal<Name>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    Exports: ResolveExportKind<Name>,
    <Exports as ResolveExportKind<Name>>::Output: ExportKindToGlobal,
{
    type Output = <<Exports as ResolveExportKind<Name>>::Output as ExportKindToGlobal>::Output;
}

impl<Module, Store, Start, Name> ResolveExportGlobal<Name> for WasmInstance<Module, Store, Start>
where
    Module: ResolveExportGlobal<Name>,
{
    type Output = <Module as ResolveExportGlobal<Name>>::Output;
}

pub trait ResolveExportTable<Name> {
    type Output;
}

impl<Funcs, Types, Exports, Name> ResolveExportTable<Name>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    Exports: ResolveExportKind<Name>,
    <Exports as ResolveExportKind<Name>>::Output: ExportKindToTable,
{
    type Output = <<Exports as ResolveExportKind<Name>>::Output as ExportKindToTable>::Output;
}

impl<Module, Store, Start, Name> ResolveExportTable<Name> for WasmInstance<Module, Store, Start>
where
    Module: ResolveExportTable<Name>,
{
    type Output = <Module as ResolveExportTable<Name>>::Output;
}

pub trait ResolveExportMemory<Name> {}

impl<Funcs, Types, Exports, Name> ResolveExportMemory<Name>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    Exports: ResolveExportKind<Name>,
    <Exports as ResolveExportKind<Name>>::Output: ExportKindToMemory,
{
}

impl<Module, Store, Start, Name> ResolveExportMemory<Name> for WasmInstance<Module, Store, Start> where
    Module: ResolveExportMemory<Name>
{
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    type ExportStart = Fn0<TTerm, tarr![OpI32Const<U7>, OpGlobalSet<U0>, OpReturn]>;
    type ExportMain = Fn0<TTerm, tarr![OpGlobalGet<U0>, OpReturn]>;
    type ExportedStateModule = WasmModule<
        TTerm,
        WasmFuncSpace<
            tarr![WasmFuncType<TTerm, TTerm>, WasmFuncType<TTerm, tarr![WasmI32Type]>],
            tarr![ExportStart, ExportMain],
        >,
        WasmModuleMemory<WasmMemoryDecl<U1, NoLimit>, TTerm>,
        WasmModuleTables<
            tarr![WasmTableDecl<U1, U1>],
            tarr![WasmElemSegment<U0, WasmConstExpr<tarr![OpI32Const<U0>]>, tarr![U1]>],
        >,
        tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U2>>],
        tarr![
            WasmExport<tstr::TS!("main"), ExportFunc<U1>>,
            WasmExport<tstr::TS!("counter"), ExportGlobal<U0>>,
            WasmExport<tstr::TS!("mem"), ExportMemory>,
            WasmExport<tstr::TS!("table"), ExportTable<U0>>
        ],
        StartFunc<U0>,
    >;

    #[test]
    fn export_accessors_read_live_state_entries_by_name() {
        type Final = ModuleProgramRun<ExportedStateModule, tarr![OpCall<U1>]>;

        assert_type_eq_all!(
            <Final as StateExportGlobal<tstr::TS!("counter")>>::Output,
            WasmGlobal<GlobalMut, WasmI32<U7>>
        );
        assert_type_eq_all!(
            <Final as StateExportMemory<tstr::TS!("mem")>>::Output,
            WasmMemory<U1, MaxPages, TTerm>
        );
        assert_type_eq_all!(
            <Final as StateExportTable<tstr::TS!("table")>>::Output,
            WasmTable<U1, U1, tarr![TableEntry<U0, U1>]>
        );
    }
}
