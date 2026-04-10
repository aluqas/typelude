use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmFunc<FuncType, LocalInits, Program>(pub PhantomData<(FuncType, LocalInits, Program)>);

impl<FuncType, LocalInits, Program> Value for WasmFunc<FuncType, LocalInits, Program> {}
