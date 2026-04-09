use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmI32<T>(pub PhantomData<T>);

impl<T> Value for WasmI32<T> {}
