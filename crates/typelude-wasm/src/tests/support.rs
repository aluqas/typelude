#![allow(dead_code, unused_imports)]

use core::ops::Add;

pub use static_assertions::assert_type_eq_all;
pub use typelude_col::{TTerm, tarr};
pub use typelude_std::core::{Evaluate, Get};
pub use tstr;
pub use typenum::{
    Const, ToUInt, U0, U1, U2, U3, U4, U5, U6, U7, U8, U9, U42, U258,
    operator_aliases::{Diff, Or, Sum},
};

pub use crate::{
    ExportFunc, ExportGlobal, ExportMemory, ExportTable, GlobalConst, GlobalMut, HostCall,
    HostCallResult, HostFuncBinding, HostGlobalBinding, HostMemoryBinding, HostTableBinding,
    ImportFunc, ImportGlobal, ImportMemory, ImportTable, InitGlobalGet, InitI32Const, InitI64Const,
    InvokeFuncWithEnv, MemoryCell, ModuleProgramRun, NoLimit, NoMemoryDecl, NoStart, RunWasm,
    StartFunc, StateExportGlobal, StateExportMemory, StateExportTable, StateTables, TableEntry,
    WasmConstExpr, WasmDataSegment, WasmElemSegment, WasmExport, WasmFunc, WasmFuncSpace,
    WasmFuncType, WasmGlobal, WasmGlobalDecl, WasmHostEnv, WasmI32, WasmI32Type, WasmI64,
    WasmI64Type, WasmImport, WasmMemArg, WasmMemory, WasmMemoryDecl, WasmModule, WasmModuleMemory,
    WasmModuleTables, WasmResolvedModule, WasmState, WasmStore, WasmTable, WasmTableDecl,
    opcode::{
        OpBlock, OpBr, OpBrIf, OpCall, OpCallIndirect, OpDrop, OpGlobalGet, OpGlobalSet,
        OpI32Add, OpI32Const, OpI32Eqz, OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8,
        OpI32Sub, OpI64Add, OpI64Const, OpI64DivS, OpI64DivU, OpI64Eq, OpI64Eqz, OpI64GtU,
        OpI64Load, OpI64LtS, OpI64Mul, OpI64RemS, OpI64Shl, OpI64ShrS, OpI64ShrU, OpI64Store,
        OpI64Sub, OpIf, OpLocalGet, OpLocalSet, OpLocalTee, OpLoop, OpMemoryGrow, OpMemorySize,
        OpReturn, OpSelect,
    },
    run::{
        StateBranches, StateGlobals, StateLocals, StateMemory, StateProgram, StateStack,
    },
};

pub type MaxPages = <Const<4294967295> as ToUInt>::Output;
pub type PageBytes = <Const<65536> as ToUInt>::Output;
pub type U9223372036854775807 = <Const<9223372036854775807usize> as ToUInt>::Output;
pub type U9223372036854775808 = <Const<9223372036854775808usize> as ToUInt>::Output;
pub type U18446744073709551615 = Or<U9223372036854775808, U9223372036854775807>;
pub type U18446744073709551614 = Diff<U18446744073709551615, U1>;
pub type U18446744073709551611 = Diff<U18446744073709551614, U3>;

pub type LittleEndian258Cells = tarr![
    MemoryCell<U3, U0>,
    MemoryCell<U2, U0>,
    MemoryCell<U1, U1>,
    MemoryCell<U0, U2>
];

pub type ZeroPages = WasmMemory<U0, MaxPages, TTerm>;
pub type OnePage = WasmMemory<U1, MaxPages, TTerm>;
pub type Store<Memory> = WasmStore<Memory, TTerm, TTerm>;
pub type Resolved<Funcs> = WasmResolvedModule<Funcs, TTerm, TTerm>;
pub type MemArg0<Offset> = WasmMemArg<U0, U0, Offset>;

pub type ModuleWithDecls<Funcs, MinPages, MaxPageLimit, DataSegments, GlobalsDecl> = WasmModule<
    TTerm,
    WasmFuncSpace<TTerm, Funcs>,
    WasmModuleMemory<WasmMemoryDecl<MinPages, MaxPageLimit>, DataSegments>,
    WasmModuleTables<TTerm, TTerm>,
    GlobalsDecl,
    TTerm,
    NoStart,
>;

pub type Module<Funcs, MinPages> = ModuleWithDecls<Funcs, MinPages, NoLimit, TTerm, TTerm>;
pub type ModuleWithGlobals<Funcs, MinPages, GlobalsDecl> =
    ModuleWithDecls<Funcs, MinPages, NoLimit, TTerm, GlobalsDecl>;
pub type ModuleWithMemoryLimits<Funcs, MinPages, MaxPageLimit> =
    ModuleWithDecls<Funcs, MinPages, MaxPageLimit, TTerm, TTerm>;
pub type ModuleWithData<Funcs, MinPages, DataSegments> =
    ModuleWithDecls<Funcs, MinPages, NoLimit, DataSegments, TTerm>;
pub type ModuleWithDataAndGlobals<Funcs, MinPages, DataSegments, GlobalsDecl> =
    ModuleWithDecls<Funcs, MinPages, NoLimit, DataSegments, GlobalsDecl>;
pub type ModuleWithTables<Funcs, Types, TablesDecl, ElemSegments = TTerm> = WasmModule<
    TTerm,
    WasmFuncSpace<Types, Funcs>,
    WasmModuleMemory<NoMemoryDecl, TTerm>,
    WasmModuleTables<TablesDecl, ElemSegments>,
    TTerm,
    TTerm,
    NoStart,
>;
pub type InitialState<Funcs, Memory, Locals, Program> =
    WasmState<Resolved<Funcs>, Store<Memory>, TTerm, Locals, TTerm, TTerm, Program>;

pub type EmptyModule = Module<TTerm, U0>;
pub type OnePageModule = Module<TTerm, U1>;
pub type Fn0<LocalDecls, Program> = WasmFunc<WasmFuncType<TTerm, TTerm>, LocalDecls, Program>;
pub type Fn1<LocalDecls, Program> =
    WasmFunc<WasmFuncType<tarr![WasmI32Type], TTerm>, LocalDecls, Program>;
pub type Fn2<LocalDecls, Program> =
    WasmFunc<WasmFuncType<tarr![WasmI32Type, WasmI32Type], TTerm>, LocalDecls, Program>;
pub type Fn1I64<LocalDecls, Program> =
    WasmFunc<WasmFuncType<tarr![WasmI64Type], tarr![WasmI64Type]>, LocalDecls, Program>;
