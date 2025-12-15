//! **Machine State**
//!
//! マシンの状態定義（Stack, Memory, CallStack, Program）

use std::marker::PhantomData;
use crate::eval::{Evaluable, Sealed};
use crate::std::trace::Trace;

/// マシンの状態
/// - `Stack`: 計算スタック (TyArray)
/// - `Locals`: ローカル変数 (TyArray)
/// - `Memory`: リニアメモリ (TyArray)
/// - `CallStack`: コールスタック (TyArray of Frames)
/// - `Program`: 現在実行中の命令列 (TyArray of Instructions)
/// - `History`: 実行履歴 (TyArray of Instructions) - Added for tracing
#[derive(Debug)]
pub struct MachineState<Stack, Locals, Memory, CallStack, Program, History = crate::std::array::TyNil>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program, History)>
);

impl<S, L, M, C, P, H> Sealed for MachineState<S, L, M, C, P, H> {}

impl<S, L, M, C, P, H> Evaluable for MachineState<S, L, M, C, P, H> {
    type Output = MachineState<S, L, M, C, P, H>;
}

impl<S, L, M, C, P, H> Trace for MachineState<S, L, M, C, P, H>
where
    S: Trace,
    L: Trace,
    M: Trace,
    H: Trace,
{
    fn fmt() -> String {
        format!(
            "MachineState {{\n  Stack: {}\n  Locals: {}\n  Memory: {}\n  History: {}\n}}",
            S::fmt(),
            L::fmt(),
            M::fmt(),
            H::fmt()
        )
    }
}
