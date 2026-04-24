use core::marker::PhantomData;

use typelude_col::{TTerm, tarr};
use typelude_wasm::{
    ModuleProgramRun, NoLimit, NoStart, StateStack, WasmElemSegment, WasmFunc, WasmFuncSpace,
    WasmFuncType, WasmI32Type, WasmMemoryDecl, WasmModule, WasmTableDecl,
    opcode::{OpCallIndirect, OpI32Const, OpReturn},
    InitI32Const,
};
use typenum::{U0, U1};

type Types = tarr![
    WasmFuncType<TTerm, tarr![WasmI32Type]>,
    WasmFuncType<tarr![WasmI32Type], tarr![WasmI32Type]>
];
type ReturnsOne =
    WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U1>, OpReturn]>;
type Funcs = tarr![ReturnsOne];
type Tables = tarr![WasmTableDecl<
    U1,
    U1,
    tarr![WasmElemSegment<U0, InitI32Const<U0>, tarr![U0]>]
>];
type Module = WasmModule<
    TTerm,
    WasmFuncSpace<Types, Funcs>,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    Tables,
    TTerm,
    TTerm,
    NoStart,
>;
type Program = tarr![OpI32Const<U0>, OpCallIndirect<U1>];
type Final = ModuleProgramRun<Module, Program>;

fn main() {
    let _: PhantomData<<Final as StateStack>::Output>;
}
