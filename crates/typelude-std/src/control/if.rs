//! 条件分岐式AST。
//!
//! 型レベルの if-then-else を表現。Cond（条件）の評価結果により、
//! Then または Else 支流を選択。

use core::marker::PhantomData;

/// 条件分岐式AST。
///
/// `Cond` の評価結果（通常は比較演算子の出力）に基づき、
/// `Then` または `Else` 支流へ分岐する型レベル構造。
///
/// セマンティクスは `Eval` impl で直接提供するか、
/// `Apply<OpIf, ...>` を通じた高階糖衣により実装可能。
/// 遅延評価や独自実行ルールが必要な場合、直接Eval実装を使用可能。
///
/// # 型パラメータ
///
/// - `Cond`: 条件式（評価により True/False等に解決）
/// - `Then`: Cond が真の場合に評価される式
/// - `Else`: Cond が偽の場合に評価される式
///
/// # 例
///
/// ```text
/// // 直接的な Eval 実装
/// impl<Then, Else> Eval for If<True, Then, Else> {
///     type Output = Then;
/// }
/// impl<Then, Else> Eval for If<False, Then, Else> {
///     type Output = Else;
/// }
///
/// // 使用
/// type Branch = Evaluate<If<True, u8, u16>>;  // Branch = u8
/// ```
pub struct If<Cond, Then, Else>(pub PhantomData<(Cond, Then, Else)>);

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::{Apply, Eval, Evaluate, If, Value, core::OpIf};

    struct Yes;
    struct No;
    struct U8Ty;
    struct U16Ty;

    impl Value for Yes {}
    impl Value for No {}
    impl Value for U8Ty {}
    impl Value for U16Ty {}

    impl<Then, Else> Eval for If<Yes, Then, Else> {
        type Output = Then;
    }

    impl<Then, Else> Eval for If<No, Then, Else> {
        type Output = Else;
    }

    #[test]
    fn if_supports_direct_eval_semantics() {
        assert_type_eq_all!(Evaluate<If<Yes, U8Ty, U16Ty>>, U8Ty);
        assert_type_eq_all!(Evaluate<If<No, U8Ty, U16Ty>>, U16Ty);
        assert_type_eq_all!(Evaluate<Apply<OpIf, (Yes, U8Ty, U16Ty)>>, U8Ty);
        assert_type_eq_all!(Evaluate<Apply<OpIf, (No, U8Ty, U16Ty)>>, U16Ty);
    }
}
