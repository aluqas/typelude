use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};

use crate::{
    helpers::name::{BoolOutput, NameEq},
    module::{
        ExportFunc, ExportGlobal, ExportMemory, ExportTable, WasmExport, WasmInstance,
        WasmResolvedModule,
    },
};

/// export 名が見つからなかったことを表す型。
pub struct MissingExport<Name>(pub PhantomData<Name>);

/// export 名から export kind を解決する helper family。
pub trait ResolveExportKind<Name> {
    /// 解決された export kind。
    type Output;
}

/// `ResolveExportKind` の再帰処理用 helper。
pub trait ResolveExportKindHelper<Name, Kind, Tail> {
    /// 途中一致か再帰継続かを反映した export kind。
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

/// export kind を関数 index に変換する helper。
pub trait ExportKindToFunc {
    /// 解決された関数 index。
    type Output;
}

impl<FuncIdx> ExportKindToFunc for ExportFunc<FuncIdx> {
    type Output = FuncIdx;
}

/// export kind を global index に変換する helper。
pub trait ExportKindToGlobal {
    /// 解決された global index。
    type Output;
}

impl<GlobalIdx> ExportKindToGlobal for ExportGlobal<GlobalIdx> {
    type Output = GlobalIdx;
}

/// export kind を table index に変換する helper。
pub trait ExportKindToTable {
    /// 解決された table index。
    type Output;
}

impl<TableIdx> ExportKindToTable for ExportTable<TableIdx> {
    type Output = TableIdx;
}

/// export kind が memory であることを確認する helper。
pub trait ExportKindToMemory {}

impl ExportKindToMemory for ExportMemory {}

/// export 名から関数 index を解決する helper。
pub trait ResolveExportFunc<Name> {
    /// 解決された関数 index。
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

/// export 名から global index を解決する helper。
pub trait ResolveExportGlobal<Name> {
    /// 解決された global index。
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

/// export 名から table index を解決する helper。
pub trait ResolveExportTable<Name> {
    /// 解決された table index。
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

/// export 名が memory を指すことを確認する helper。
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
            WasmExport<typelude_str::tstr!("main"), ExportFunc<U1>>,
            WasmExport<typelude_str::tstr!("counter"), ExportGlobal<U0>>,
            WasmExport<typelude_str::tstr!("mem"), ExportMemory>,
            WasmExport<typelude_str::tstr!("table"), ExportTable<U0>>
        ],
        StartFunc<U0>,
    >;

    #[test]
    fn export_accessors_read_live_state_entries_by_name() {
        type Final = ModuleProgramRun<ExportedStateModule, tarr![OpCall<U1>]>;

        assert_type_eq_all!(
            <Final as StateExportGlobal<typelude_str::tstr!("counter")>>::Output,
            WasmGlobal<GlobalMut, WasmI32<U7>>
        );
        assert_type_eq_all!(
            <Final as StateExportMemory<typelude_str::tstr!("mem")>>::Output,
            WasmMemory<U1, MaxPages, TTerm>
        );
        assert_type_eq_all!(
            <Final as StateExportTable<typelude_str::tstr!("table")>>::Output,
            WasmTable<U1, U1, tarr![TableEntry<U0, U1>]>
        );
    }
}
