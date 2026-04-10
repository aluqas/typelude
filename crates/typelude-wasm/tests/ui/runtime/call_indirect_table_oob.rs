use core::marker::PhantomData;

use typelude_col::{TTerm, tarr};
use typelude_wasm::{
    ModuleProgramRun, NoLimit, NoStart, StateStack, WasmFuncSpace, WasmFuncType, WasmI32Type,
    WasmMemoryDecl, WasmModule, WasmTableDecl,
    opcode::{OpCallIndirect, OpI32Const},
};
use typenum::{U0, U1};

type Types = tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>];
type Tables = tarr![WasmTableDecl<U1, U1, TTerm>];
type Module = WasmModule<
    TTerm,
    WasmFuncSpace<Types, TTerm>,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    Tables,
    TTerm,
    TTerm,
    NoStart,
>;
type Program = tarr![OpI32Const<U1>, OpCallIndirect<U0>];
type Final = ModuleProgramRun<Module, Program>;

fn main() {
    let _: PhantomData<<Final as StateStack>::Output>;
}
