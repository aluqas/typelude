use typelude_col::TArr;

use crate::{
    helpers::name::{BoolOutput, NameEq},
    module::{ExportFunc, WasmExport, WasmResolvedModule},
};

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

impl<Name, ExportName, Kind, Tail> ResolveExportKind<Name> for TArr<WasmExport<ExportName, Kind>, Tail>
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

pub trait ExportKindToFunc {
    type Output;
}

impl<FuncIdx> ExportKindToFunc for ExportFunc<FuncIdx> {
    type Output = FuncIdx;
}

pub trait ResolveExportFunc<Name> {
    type Output;
}

impl<Funcs, Types, Exports, Name> ResolveExportFunc<Name> for WasmResolvedModule<Funcs, Types, Exports>
where
    Exports: ResolveExportKind<Name>,
    <Exports as ResolveExportKind<Name>>::Output: ExportKindToFunc,
{
    type Output = <<Exports as ResolveExportKind<Name>>::Output as ExportKindToFunc>::Output;
}
