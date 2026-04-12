//! ループ構造式AST。
//!
//! 型レベルの while ループを表現。状態 State の遷移を
//! 述語 Pred が真である限り繰り返す。

use core::marker::PhantomData;

/// While ループ構造式AST。
///
/// `Pred`（述語）が真である限り、`Step`（遷移）を繰り返し適用し、
/// `State`（状態）を更新する型レベル制御構造。
///
/// `Pred` と `Step` は一級演算子として機能するよう期待される。
/// 直接的な Eval 実装も許可され、遅延評価や独自セマンティクス
/// （例：特定ドメインの実行ルール）が必要時に利用可能。
///
/// # 型パラメータ
///
/// - `Pred`: 述語演算子（状態に対して真偽を判定）
/// - `Step`: 遷移演算子（状態を新しい状態へ変換）
/// - `State`: ループの初期・中間・最終状態
///
/// # 例
///
/// ```text
/// // 直接的な Eval 実装：述語が偽の場合、状態をそのまま返す
/// impl<Step, State> Eval for While<False, Step, State> {
///     type Output = State;
/// }
///
/// // 使用
/// type Result = Evaluate<While<False, SomeStep, u8>>;  // Result = u8
/// ```
pub struct While<Pred, Step, State>(pub PhantomData<(Pred, Step, State)>);

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::{Eval, Evaluate, While};

    struct Halt;
    struct Step;

    impl<State> Eval for While<Halt, Step, State> {
        type Output = State;
    }

    #[test]
    fn while_supports_direct_eval_semantics() {
        assert_type_eq_all!(Evaluate<While<Halt, Step, u8>>, u8);
    }
}
