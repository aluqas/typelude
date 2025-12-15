//! **Machine State**
//!
//! マシンの状態定義（Stack, Memory, CallStack, Program）

use std::marker::PhantomData;

use crate::eval::{Evaluable, Sealed};

/// マシンの状態
/// - `Stack`: 計算スタック (TyArray)
/// - `Locals`: ローカル変数 (TyArray)
/// - `Memory`: リニアメモリ (TyArray)
/// - `CallStack`: コールスタック (TyArray of Frames)
/// - `Program`: 現在実行中の命令列 (TyArray of Instructions)
#[derive(Debug)]
pub struct MachineState<Stack, Locals, Memory, CallStack, Program>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program)>,
);

impl<S, L, M, C, P> Sealed for MachineState<S, L, M, C, P> {}

impl<S, L, M, C, P> Evaluable for MachineState<S, L, M, C, P> {
    type Output = MachineState<S, L, M, C, P>;
}
