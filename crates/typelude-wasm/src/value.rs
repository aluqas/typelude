use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmI32<T>(PhantomData<T>);

impl<T> Value for WasmI32<T> {}

#[derive(Debug, Default)]
pub struct WasmI32Type;

impl Value for WasmI32Type {}

#[derive(Debug, Default)]
pub struct WasmI64<T>(PhantomData<T>);

impl<T> Value for WasmI64<T> {}

#[derive(Debug, Default)]
pub struct WasmI64Type;

impl Value for WasmI64Type {}

#[derive(Debug, Default)]
pub struct WasmF32<T>(PhantomData<T>);

impl<T> Value for WasmF32<T> {}

#[derive(Debug, Default)]
pub struct WasmF32Type;

impl Value for WasmF32Type {}

#[derive(Debug, Default)]
pub struct WasmF64<T>(PhantomData<T>);

impl<T> Value for WasmF64<T> {}

#[derive(Debug, Default)]
pub struct WasmF64Type;

impl Value for WasmF64Type {}
