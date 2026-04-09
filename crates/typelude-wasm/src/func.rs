use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmFunc<ParamCount, LocalInits, Program>(
    pub PhantomData<(ParamCount, LocalInits, Program)>,
);

impl<ParamCount, LocalInits, Program> Value for WasmFunc<ParamCount, LocalInits, Program> {}
