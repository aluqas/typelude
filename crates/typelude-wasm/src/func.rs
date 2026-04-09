use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmFunc<Locals, Program>(pub PhantomData<(Locals, Program)>);

impl<Locals, Program> Value for WasmFunc<Locals, Program> {}
