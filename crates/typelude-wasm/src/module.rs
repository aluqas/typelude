use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmModule<Imports, Funcs, Memory, Tables, Globals, Exports, Start>(
    PhantomData<(Imports, Funcs, Memory, Tables, Globals, Exports, Start)>,
);

impl<Imports, Funcs, Memory, Tables, Globals, Exports, Start> Value
    for WasmModule<Imports, Funcs, Memory, Tables, Globals, Exports, Start>
{
}

#[derive(Debug, Default)]
pub struct WasmResolvedModule<Funcs, Types, Exports>(PhantomData<(Funcs, Types, Exports)>);

impl<Funcs, Types, Exports> Value for WasmResolvedModule<Funcs, Types, Exports> {}

#[derive(Debug, Default)]
pub struct WasmInstance<Module, Store, Start = NoStart>(PhantomData<(Module, Store, Start)>);

impl<Module, Store, Start> Value for WasmInstance<Module, Store, Start> {}

#[derive(Debug, Default)]
pub struct WasmFuncSpace<Types, Funcs>(PhantomData<(Types, Funcs)>);

impl<Types, Funcs> Value for WasmFuncSpace<Types, Funcs> {}

#[derive(Debug, Default)]
pub struct WasmFuncType<Params, Results>(PhantomData<(Params, Results)>);

impl<Params, Results> Value for WasmFuncType<Params, Results> {}

#[derive(Debug, Default)]
pub struct WasmModuleMemory<Decl, DataSegments>(PhantomData<(Decl, DataSegments)>);

impl<Decl, DataSegments> Value for WasmModuleMemory<Decl, DataSegments> {}

#[derive(Debug, Default)]
pub struct WasmMemoryDecl<MinPages, MaxPages>(PhantomData<(MinPages, MaxPages)>);

impl<MinPages, MaxPages> Value for WasmMemoryDecl<MinPages, MaxPages> {}

#[derive(Debug, Default)]
pub struct NoMemoryDecl;

impl Value for NoMemoryDecl {}

#[derive(Debug, Default)]
pub struct WasmModuleTables<Decls, ElemSegments>(PhantomData<(Decls, ElemSegments)>);

impl<Decls, ElemSegments> Value for WasmModuleTables<Decls, ElemSegments> {}

#[derive(Debug, Default)]
pub struct WasmTableDecl<Min, Max>(PhantomData<(Min, Max)>);

impl<Min, Max> Value for WasmTableDecl<Min, Max> {}

#[derive(Debug, Default)]
pub struct WasmGlobalDecl<Mutability, InitExpr>(PhantomData<(Mutability, InitExpr)>);

impl<Mutability, InitExpr> Value for WasmGlobalDecl<Mutability, InitExpr> {}

#[derive(Debug, Default)]
pub struct WasmImport<ModuleName, FieldName, Kind>(PhantomData<(ModuleName, FieldName, Kind)>);

impl<ModuleName, FieldName, Kind> Value for WasmImport<ModuleName, FieldName, Kind> {}

#[derive(Debug, Default)]
pub struct WasmExport<Name, Kind>(PhantomData<(Name, Kind)>);

impl<Name, Kind> Value for WasmExport<Name, Kind> {}

#[derive(Debug, Default)]
pub struct WasmHostEnv<Funcs, Globals, Memory, Tables>(
    PhantomData<(Funcs, Globals, Memory, Tables)>,
);

impl<Funcs, Globals, Memory, Tables> Value for WasmHostEnv<Funcs, Globals, Memory, Tables> {}

#[derive(Debug, Default)]
pub struct HostFuncBinding<ModuleName, FieldName, Host>(
    PhantomData<(ModuleName, FieldName, Host)>,
);

impl<ModuleName, FieldName, Host> Value for HostFuncBinding<ModuleName, FieldName, Host> {}

#[derive(Debug, Default)]
pub struct HostGlobalBinding<ModuleName, FieldName, Global>(
    PhantomData<(ModuleName, FieldName, Global)>,
);

impl<ModuleName, FieldName, Global> Value for HostGlobalBinding<ModuleName, FieldName, Global> {}

#[derive(Debug, Default)]
pub struct HostMemoryBinding<ModuleName, FieldName, Memory>(
    PhantomData<(ModuleName, FieldName, Memory)>,
);

impl<ModuleName, FieldName, Memory> Value for HostMemoryBinding<ModuleName, FieldName, Memory> {}

#[derive(Debug, Default)]
pub struct HostTableBinding<ModuleName, FieldName, Table>(
    PhantomData<(ModuleName, FieldName, Table)>,
);

impl<ModuleName, FieldName, Table> Value for HostTableBinding<ModuleName, FieldName, Table> {}

#[derive(Debug, Default)]
pub struct ImportFunc<FuncType>(PhantomData<FuncType>);

impl<FuncType> Value for ImportFunc<FuncType> {}

#[derive(Debug, Default)]
pub struct ImportGlobal<Mutability, ValueType>(PhantomData<(Mutability, ValueType)>);

impl<Mutability, ValueType> Value for ImportGlobal<Mutability, ValueType> {}

#[derive(Debug, Default)]
pub struct ImportMemory<MinPages, MaxPages>(PhantomData<(MinPages, MaxPages)>);

impl<MinPages, MaxPages> Value for ImportMemory<MinPages, MaxPages> {}

#[derive(Debug, Default)]
pub struct ImportTable<Min, Max>(PhantomData<(Min, Max)>);

impl<Min, Max> Value for ImportTable<Min, Max> {}

#[derive(Debug, Default)]
pub struct ExportFunc<FuncIdx>(PhantomData<FuncIdx>);

impl<FuncIdx> Value for ExportFunc<FuncIdx> {}

#[derive(Debug, Default)]
pub struct ExportGlobal<GlobalIdx>(PhantomData<GlobalIdx>);

impl<GlobalIdx> Value for ExportGlobal<GlobalIdx> {}

#[derive(Debug, Default)]
pub struct ExportTable<TableIdx>(PhantomData<TableIdx>);

impl<TableIdx> Value for ExportTable<TableIdx> {}

#[derive(Debug, Default)]
pub struct ExportMemory;

impl Value for ExportMemory {}

#[derive(Debug, Default)]
pub struct NoStart;

impl Value for NoStart {}

#[derive(Debug, Default)]
pub struct StartFunc<FuncIdx>(PhantomData<FuncIdx>);

impl<FuncIdx> Value for StartFunc<FuncIdx> {}

#[derive(Debug, Default)]
pub struct WasmConstExpr<Instrs>(PhantomData<Instrs>);

impl<Instrs> Value for WasmConstExpr<Instrs> {}

#[derive(Debug, Default)]
pub struct WasmMemArg<MemoryIdx, Align, Offset>(PhantomData<(MemoryIdx, Align, Offset)>);

impl<MemoryIdx, Align, Offset> Value for WasmMemArg<MemoryIdx, Align, Offset> {}

#[derive(Debug, Default)]
pub struct InitI32Const<ValueT>(PhantomData<ValueT>);

impl<ValueT> Value for InitI32Const<ValueT> {}

#[derive(Debug, Default)]
pub struct InitI64Const<ValueT>(PhantomData<ValueT>);

impl<ValueT> Value for InitI64Const<ValueT> {}

#[derive(Debug, Default)]
pub struct InitGlobalGet<GlobalIdx>(PhantomData<GlobalIdx>);

impl<GlobalIdx> Value for InitGlobalGet<GlobalIdx> {}

#[derive(Debug, Default)]
pub struct WasmDataSegment<OffsetExpr, Bytes>(PhantomData<(OffsetExpr, Bytes)>);

impl<OffsetExpr, Bytes> Value for WasmDataSegment<OffsetExpr, Bytes> {}

#[derive(Debug, Default)]
pub struct WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>(
    PhantomData<(TableIdx, OffsetExpr, FuncIndices)>,
);

impl<TableIdx, OffsetExpr, FuncIndices> Value
    for WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>
{
}

#[derive(Debug, Default)]
pub struct WasmHostFunc<FuncType, Host>(PhantomData<(FuncType, Host)>);

impl<FuncType, Host> Value for WasmHostFunc<FuncType, Host> {}

#[derive(Debug, Default)]
pub struct GlobalConst;

impl Value for GlobalConst {}

#[derive(Debug, Default)]
pub struct GlobalMut;

impl Value for GlobalMut {}

#[derive(Debug, Default)]
pub struct NoLimit;

impl Value for NoLimit {}
