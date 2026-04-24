use core::marker::PhantomData;

use typelude_col::{TTerm, tarr};
use typelude_wasm::{
    GlobalConst, InitI32Const, ModuleProgramRun, NoLimit, NoStart, StateStack, WasmFuncSpace,
    WasmGlobalDecl, WasmMemoryDecl, WasmModule, opcode::{OpGlobalSet, OpI32Const},
};
use typenum::U0;

type Module = WasmModule<
    TTerm,
    WasmFuncSpace<TTerm, TTerm>,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    TTerm,
    tarr![WasmGlobalDecl<GlobalConst, InitI32Const<U0>>],
    TTerm,
    NoStart,
>;

type Final = ModuleProgramRun<Module, tarr![OpI32Const<U0>, OpGlobalSet<U0>]>;

fn main() {
    let _: PhantomData<<Final as StateStack>::Output>;
}
