//! **Machine State**
//!
//! マシンの状態定義（Stack, Memory, CallStack, Program）

use std::marker::PhantomData;
use crate::eval::{Evaluable, Sealed};

/// マシンの状態
/// - `Stack`: 計算スタック (TyArray)
/// - `Memory`: リニアメモリ (TyArray)
/// - `CallStack`: コールスタック (TyArray of Programs)
/// - `Program`: 現在実行中の命令列 (TyArray of Instructions)
#[derive(Debug)]
pub struct MachineState<Stack, Memory, CallStack, Program>(
    PhantomData<(Stack, Memory, CallStack, Program)>
);

impl<S, M, C, P> Sealed for MachineState<S, M, C, P> {}

impl<S, M, C, P> Evaluable for MachineState<S, M, C, P> {
    type Output = MachineState<S, M, C, P>;
}
