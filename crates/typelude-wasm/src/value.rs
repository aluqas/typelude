use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmI32<T>(pub PhantomData<T>);

impl<T> Value for WasmI32<T> {}

#[derive(Debug, Default)]
pub struct WasmI32Type;

impl Value for WasmI32Type {}
