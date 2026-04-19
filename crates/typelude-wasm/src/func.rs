use core::marker::PhantomData;

use typelude_std::core::Value;

/// WebAssembly 関数定義。
///
/// `FuncType` はシグネチャ、`LocalDecls` は追加 local の型宣言列、
/// `Program` は関数本体の opcode 列です。module の function space に格納され、
/// `OpCall` / `OpCallIndirect` から参照されます。
#[derive(Debug, Default)]
pub struct WasmFunc<FuncType, LocalDecls, Program>(
    pub PhantomData<(FuncType, LocalDecls, Program)>,
);

impl<FuncType, LocalDecls, Program> Value for WasmFunc<FuncType, LocalDecls, Program> {}
