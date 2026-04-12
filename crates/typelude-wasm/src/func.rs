use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmFunc<FuncType, LocalDecls, Program>(pub PhantomData<(FuncType, LocalDecls, Program)>);

impl<FuncType, LocalDecls, Program> Value for WasmFunc<FuncType, LocalDecls, Program> {}
