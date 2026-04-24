use core::marker::PhantomData;

use typelude_std::core::Value;

/// WebAssembly モジュール全体を表す型。
///
/// `twat!` が最終的に生成するのもこの型です。imports, 関数空間,
/// memory/table/global 宣言, exports, start 関数を型引数に保持します。
/// これはまだ実行可能な store を持たない「宣言」の形であり、`InstantiateModule`
/// を通ることで `WasmInstance` に変換されます。
///
/// 型引数は WASM section に近い順序で並びます。`Funcs` は `WasmFuncSpace`、
/// `Memory` は `WasmModuleMemory`、`Tables` は `WasmModuleTables`
/// を想定します。
#[derive(Debug, Default)]
pub struct WasmModule<Imports, Funcs, Memory, Tables, Globals, Exports, Start>(
    PhantomData<(Imports, Funcs, Memory, Tables, Globals, Exports, Start)>,
);

impl<Imports, Funcs, Memory, Tables, Globals, Exports, Start> Value
    for WasmModule<Imports, Funcs, Memory, Tables, Globals, Exports, Start>
{
}

/// インスタンス化後に実行系が参照する解決済みモジュール。
///
/// import 解決や start 実行準備の後、runtime はこの型を `WasmState`
/// に載せて進みます。 `Funcs` には import
/// 関数と定義済み関数を連結した関数空間が入り、 `Types` には `call_indirect`
/// の型 index 解決に使う関数型一覧が入ります。 `Exports` は name-based accessor
/// と `InvokeExport*` が参照します。
#[derive(Debug, Default)]
pub struct WasmResolvedModule<Funcs, Types, Exports>(PhantomData<(Funcs, Types, Exports)>);

impl<Funcs, Types, Exports> Value for WasmResolvedModule<Funcs, Types, Exports> {}

/// store と start 情報を束ねたインスタンス化済みモジュール。
///
/// `Module` は `WasmResolvedModule`、`Store` は `WasmStore`、`Start` は
/// `NoStart` または `StartFunc<FuncIdx>` です。`BuildInvokeState` や
/// `BuildProgramState` はこの型から初期 `WasmState` を構築します。
#[derive(Debug, Default)]
pub struct WasmInstance<Module, Store, Start = NoStart>(PhantomData<(Module, Store, Start)>);

impl<Module, Store, Start> Value for WasmInstance<Module, Store, Start> {}

/// 関数シグネチャ空間と関数本体空間。
///
/// `Types` は関数型の一覧、`Funcs` は import を含めた関数定義の一覧です。
/// インスタンス化時には import
/// 関数列が先頭に入り、その後に定義済み関数列が続きます。 そのため WASM の
/// function index と型リスト上の index が一致する前提で lookup できます。
#[derive(Debug, Default)]
pub struct WasmFuncSpace<Types, Funcs>(PhantomData<(Types, Funcs)>);

impl<Types, Funcs> Value for WasmFuncSpace<Types, Funcs> {}

/// WebAssembly の関数型。
///
/// `Params` と `Results` はどちらも typelude の型リストです。
/// value type は `WasmI32Type` / `WasmI64Type` / `WasmF32Type` / `WasmF64Type`
/// で表します。 `FuncTypeEq` や `PopArgs`
/// はこの型を基準に引数・戻り値の整合性を見ます。
#[derive(Debug, Default)]
pub struct WasmFuncType<Params, Results>(PhantomData<(Params, Results)>);

impl<Params, Results> Value for WasmFuncType<Params, Results> {}

/// モジュールの memory 宣言と data segment 群。
///
/// `Decl` は `WasmMemoryDecl` または `NoMemoryDecl`、`DataSegments` は
/// `WasmDataSegment` の型リストです。インスタンス化時に
/// `MaterializeMemoryStore` が store 上の `WasmMemory` へ変換します。
#[derive(Debug, Default)]
pub struct WasmModuleMemory<Decl, DataSegments>(PhantomData<(Decl, DataSegments)>);

impl<Decl, DataSegments> Value for WasmModuleMemory<Decl, DataSegments> {}

/// 線形メモリの宣言。
///
/// `MinPages` は必須、`MaxPages` は `NoLimit` か上限ページ数です。
/// store 上では `NoLimit` は実装上の最大ページ数へ正規化されます。
/// 現状の frontend validation では single-memory のみを受理します。
#[derive(Debug, Default)]
pub struct WasmMemoryDecl<MinPages, MaxPages>(PhantomData<(MinPages, MaxPages)>);

impl<MinPages, MaxPages> Value for WasmMemoryDecl<MinPages, MaxPages> {}

/// memory section を持たないモジュールを表す番兵型。
#[derive(Debug, Default)]
pub struct NoMemoryDecl;

impl Value for NoMemoryDecl {}

/// モジュールの table 宣言と elem segment 群。
///
/// `Decls` は `WasmTableDecl` の型リスト、`ElemSegments` は `WasmElemSegment`
/// の型リストです。インスタンス化時に import table と定義済み table を結合し、
/// elem segment を書き込みます。
#[derive(Debug, Default)]
pub struct WasmModuleTables<Decls, ElemSegments>(PhantomData<(Decls, ElemSegments)>);

impl<Decls, ElemSegments> Value for WasmModuleTables<Decls, ElemSegments> {}

/// 単一 table 宣言。
///
/// 現在の runtime は funcref table を前提にし、entries は store 上の
/// `WasmTable` に `TableEntry<SlotIdx, FuncIdx>` として materialize されます。
#[derive(Debug, Default)]
pub struct WasmTableDecl<Min, Max>(PhantomData<(Min, Max)>);

impl<Min, Max> Value for WasmTableDecl<Min, Max> {}

/// グローバル変数宣言。
///
/// `Mutability` には `GlobalConst` か `GlobalMut`、
/// `InitExpr` には const expr の型表現が入ります。
#[derive(Debug, Default)]
pub struct WasmGlobalDecl<Mutability, InitExpr>(PhantomData<(Mutability, InitExpr)>);

impl<Mutability, InitExpr> Value for WasmGlobalDecl<Mutability, InitExpr> {}

/// import エントリ。
///
/// `ModuleName` と `FieldName` は型レベル文字列、`Kind` は import 種別です。
/// import 解決では `WasmHostEnv` 内の binding と module/name の両方を照合し、
/// kind ごとの compat trait で型や制限の整合性を確認します。
#[derive(Debug, Default)]
pub struct WasmImport<ModuleName, FieldName, Kind>(PhantomData<(ModuleName, FieldName, Kind)>);

impl<ModuleName, FieldName, Kind> Value for WasmImport<ModuleName, FieldName, Kind> {}

/// export エントリ。
///
/// `Name` は型レベル文字列、`Kind` は `ExportFunc` / `ExportGlobal` /
/// `ExportTable` / `ExportMemory` のいずれかです。runtime の name-based API は
/// このリストを線形探索します。
#[derive(Debug, Default)]
pub struct WasmExport<Name, Kind>(PhantomData<(Name, Kind)>);

impl<Name, Kind> Value for WasmExport<Name, Kind> {}

/// ホスト側から与える import 解決環境。
///
/// 関数・グローバル・メモリ・テーブルの binding 一覧を保持します。
/// import を持たない module では `EmptyHostEnv` を使います。binding は section
/// ごとに 別リストとして渡し、インスタンス化時に必要な import kind
/// だけが参照されます。
#[derive(Debug, Default)]
pub struct WasmHostEnv<Funcs, Globals, Memory, Tables>(
    PhantomData<(Funcs, Globals, Memory, Tables)>,
);

impl<Funcs, Globals, Memory, Tables> Value for WasmHostEnv<Funcs, Globals, Memory, Tables> {}

/// ホスト関数 import の binding。
#[derive(Debug, Default)]
pub struct HostFuncBinding<ModuleName, FieldName, Host>(
    PhantomData<(ModuleName, FieldName, Host)>,
);

impl<ModuleName, FieldName, Host> Value for HostFuncBinding<ModuleName, FieldName, Host> {}

/// ホストグローバル import の binding。
#[derive(Debug, Default)]
pub struct HostGlobalBinding<ModuleName, FieldName, Global>(
    PhantomData<(ModuleName, FieldName, Global)>,
);

impl<ModuleName, FieldName, Global> Value for HostGlobalBinding<ModuleName, FieldName, Global> {}

/// ホストメモリ import の binding。
#[derive(Debug, Default)]
pub struct HostMemoryBinding<ModuleName, FieldName, Memory>(
    PhantomData<(ModuleName, FieldName, Memory)>,
);

impl<ModuleName, FieldName, Memory> Value for HostMemoryBinding<ModuleName, FieldName, Memory> {}

/// ホストテーブル import の binding。
#[derive(Debug, Default)]
pub struct HostTableBinding<ModuleName, FieldName, Table>(
    PhantomData<(ModuleName, FieldName, Table)>,
);

impl<ModuleName, FieldName, Table> Value for HostTableBinding<ModuleName, FieldName, Table> {}

/// 関数 import の kind。
#[derive(Debug, Default)]
pub struct ImportFunc<FuncType>(PhantomData<FuncType>);

impl<FuncType> Value for ImportFunc<FuncType> {}

/// グローバル import の kind。
#[derive(Debug, Default)]
pub struct ImportGlobal<Mutability, ValueType>(PhantomData<(Mutability, ValueType)>);

impl<Mutability, ValueType> Value for ImportGlobal<Mutability, ValueType> {}

/// メモリ import の kind。
#[derive(Debug, Default)]
pub struct ImportMemory<MinPages, MaxPages>(PhantomData<(MinPages, MaxPages)>);

impl<MinPages, MaxPages> Value for ImportMemory<MinPages, MaxPages> {}

/// テーブル import の kind。
#[derive(Debug, Default)]
pub struct ImportTable<Min, Max>(PhantomData<(Min, Max)>);

impl<Min, Max> Value for ImportTable<Min, Max> {}

/// 関数 export の kind。
#[derive(Debug, Default)]
pub struct ExportFunc<FuncIdx>(PhantomData<FuncIdx>);

impl<FuncIdx> Value for ExportFunc<FuncIdx> {}

/// グローバル export の kind。
#[derive(Debug, Default)]
pub struct ExportGlobal<GlobalIdx>(PhantomData<GlobalIdx>);

impl<GlobalIdx> Value for ExportGlobal<GlobalIdx> {}

/// テーブル export の kind。
#[derive(Debug, Default)]
pub struct ExportTable<TableIdx>(PhantomData<TableIdx>);

impl<TableIdx> Value for ExportTable<TableIdx> {}

/// メモリ export の kind。
#[derive(Debug, Default)]
pub struct ExportMemory;

impl Value for ExportMemory {}

/// start 関数を持たないモジュールを表す番兵型。
#[derive(Debug, Default)]
pub struct NoStart;

impl Value for NoStart {}

/// start 関数の index。
#[derive(Debug, Default)]
pub struct StartFunc<FuncIdx>(PhantomData<FuncIdx>);

impl<FuncIdx> Value for StartFunc<FuncIdx> {}

/// const expression の命令列。
#[derive(Debug, Default)]
pub struct WasmConstExpr<Instrs>(PhantomData<Instrs>);

impl<Instrs> Value for WasmConstExpr<Instrs> {}

/// memory load/store 系の memarg。
///
/// 現状の runtime / frontend validation では `MemoryIdx = 0` のみを受理します。
/// `Align` は IR として保持しますが、現時点の runtime
/// 意味論は実効アドレス計算に `Offset` だけを使います。実効アドレスは stack
/// 上の base address と `Offset` の和です。
#[derive(Debug, Default)]
pub struct WasmMemArg<MemoryIdx, Align, Offset>(PhantomData<(MemoryIdx, Align, Offset)>);

impl<MemoryIdx, Align, Offset> Value for WasmMemArg<MemoryIdx, Align, Offset> {}

/// `i32.const` による初期化式。
#[derive(Debug, Default)]
pub struct InitI32Const<ValueT>(PhantomData<ValueT>);

impl<ValueT> Value for InitI32Const<ValueT> {}

/// `i64.const` による初期化式。
#[derive(Debug, Default)]
pub struct InitI64Const<ValueT>(PhantomData<ValueT>);

impl<ValueT> Value for InitI64Const<ValueT> {}

/// `global.get` による初期化式。
#[derive(Debug, Default)]
pub struct InitGlobalGet<GlobalIdx>(PhantomData<GlobalIdx>);

impl<GlobalIdx> Value for InitGlobalGet<GlobalIdx> {}

/// data segment。
///
/// `OffsetExpr` は `i32.const` または immutable global 参照を想定し、
/// `Bytes` は byte の型リストです。インスタンス化時に memory へ書き込まれます。
#[derive(Debug, Default)]
pub struct WasmDataSegment<OffsetExpr, Bytes>(PhantomData<(OffsetExpr, Bytes)>);

impl<OffsetExpr, Bytes> Value for WasmDataSegment<OffsetExpr, Bytes> {}

/// elem segment。
///
/// `TableIdx` が対象 table、`OffsetExpr` が開始 slot、`FuncIndices`
/// が書き込む関数 index 列です。 checked `call_indirect` はここで materialize
/// された table entry を参照します。
#[derive(Debug, Default)]
pub struct WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>(
    PhantomData<(TableIdx, OffsetExpr, FuncIndices)>,
);

impl<TableIdx, OffsetExpr, FuncIndices> Value
    for WasmElemSegment<TableIdx, OffsetExpr, FuncIndices>
{
}

/// ホスト実装を伴う関数定義。
#[derive(Debug, Default)]
pub struct WasmHostFunc<FuncType, Host>(PhantomData<(FuncType, Host)>);

impl<FuncType, Host> Value for WasmHostFunc<FuncType, Host> {}

/// immutable global を表す mutability マーカー。
#[derive(Debug, Default)]
pub struct GlobalConst;

impl Value for GlobalConst {}

/// mutable global を表す mutability マーカー。
#[derive(Debug, Default)]
pub struct GlobalMut;

impl Value for GlobalMut {}

/// 上限なしを表すマーカー。
#[derive(Debug, Default)]
pub struct NoLimit;

impl Value for NoLimit {}
