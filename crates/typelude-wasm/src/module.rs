use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmModule<Funcs, InitialMemory>(pub PhantomData<(Funcs, InitialMemory)>);

impl<Funcs, InitialMemory> Value for WasmModule<Funcs, InitialMemory> {}
