use core::marker::PhantomData;

use typelude_std::core::Value;

/// `i32` 値。
///
/// `T` は typenum による 32bit bit-pattern 表現です。
#[derive(Debug, Default)]
pub struct WasmI32<T>(PhantomData<T>);

impl<T> Value for WasmI32<T> {}

/// `i32` の値型タグ。
#[derive(Debug, Default)]
pub struct WasmI32Type;

impl Value for WasmI32Type {}

/// `i64` 値。
///
/// `T` は typenum による 64bit bit-pattern 表現です。
#[derive(Debug, Default)]
pub struct WasmI64<T>(PhantomData<T>);

impl<T> Value for WasmI64<T> {}

/// `i64` の値型タグ。
#[derive(Debug, Default)]
pub struct WasmI64Type;

impl Value for WasmI64Type {}

/// `f32` 値。
///
/// 現状は主に reinterpret 用のビット列コンテナとして使います。
#[derive(Debug, Default)]
pub struct WasmF32<T>(PhantomData<T>);

impl<T> Value for WasmF32<T> {}

/// `f32` の値型タグ。
#[derive(Debug, Default)]
pub struct WasmF32Type;

impl Value for WasmF32Type {}

/// `f64` 値。
///
/// 現状は主に reinterpret 用のビット列コンテナとして使います。
#[derive(Debug, Default)]
pub struct WasmF64<T>(PhantomData<T>);

impl<T> Value for WasmF64<T> {}

/// `f64` の値型タグ。
#[derive(Debug, Default)]
pub struct WasmF64Type;

impl Value for WasmF64Type {}
