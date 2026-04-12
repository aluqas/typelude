use core::marker::PhantomData;

use crate::core::Op;

/// 型レベルモナドインターフェース。
///
/// `Pure<F, A>` と `Bind<F, MA, K>` はモナド種（モナド型コンストラクタ）`F`から
/// モナド式を構築するためのキャノニカルなエイリアス。
/// 不純なエフェクト（状態、例外、読み込み環境等）を型安全に表現する。
///
/// # 型パラメータ
///
/// - `F`: モナド種（種 * -> * の型）
/// - `A`: モナド的値の戻り値型
/// - `MA`: モナド的計算（Monadの計算式）
/// - `K`: 継続演算子（MA の結果を受け取り新たなモナド計算を返す Op）
///
/// # 例
///
/// ```text
/// // 恒等モナド IdK 上での計算合成
/// struct Duplicate;
/// impl Op<Unit> for Duplicate {
///     type Output = Pure<IdK, Pair<Unit, Unit>>;  // Unit を複製する操作
/// }
///
/// type Program = Bind<IdK, Pure<IdK, Unit>, Duplicate>;
/// // Program 実行 -> Duplicate により Unit, Unit ペアを返す
/// ```
pub trait Monad {
    /// 値から純粋なモナド計算を構築する。
    type Pure<A>;
    /// モナド計算をバインド（合成）する。
    type Bind<MA, K>;
}

pub type Pure<F, A> = <F as Monad>::Pure<A>;
pub type Bind<F, MA, K> = <F as Monad>::Bind<MA, K>;

/// モナド間の変換インターフェース。
///
/// 外側モナド F に内側モナド Inner を積み上げ、
/// Inner の計算を F へリフトする。
pub trait MonadTrans<Inner>: Monad {
    /// 内側の計算を外側へリフトする（埋め込む）。
    type Lift<MA>;
}

/// 読み込み環境を扱うモナドインターフェース。
///
/// Reader エフェクト：環境依存の計算。
pub trait MonadReader<R>: Monad {
    /// 現在の環境を問い合わせる操作。
    type Ask;
    /// 計算を別の環境で実行する（部分的に切り出す）。
    type Local<F, MA>;
}

/// 状態を扱うモナドインターフェース。
///
/// State エフェクト：状態依存の計算。
pub trait MonadState<S>: Monad {
    /// 現在の状態を取得する操作。
    type Get;
    /// 状態を新たな値に置き換える操作。
    type Put<NewState>;
    /// 状態に関数を適用して変更する操作。
    type Modify<F>;
}

pub trait MonadWriter<W>: Monad {
    type Tell<Chunk>;
    type Listen<MA>;
    type Censor<F, MA>;
}

pub trait MonadError<E>: Monad {
    type Throw<Reason>;
    type Catch<MA, H>;
}

pub trait MonadSuspend<R>: Monad {
    type Suspend<Request>;
}

pub trait Empty {
    type Output;
}

pub trait Append<Rhs> {
    type Output;
}

pub struct LConst<T>(pub PhantomData<T>);

impl<A, T> Op<A> for LConst<T> {
    type Output = T;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::{
        core::Evaluate,
        effect::{IdK, Pair, RunId, Unit},
    };

    struct Duplicate;

    impl Op<Unit> for Duplicate {
        type Output = Pure<IdK, Pair<Unit, Unit>>;
    }

    #[test]
    fn monad_usage_example_builds_id_program() {
        type Program = Bind<IdK, Pure<IdK, Unit>, Duplicate>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }

    #[test]
    fn lconst_can_be_used_as_bind_continuation() {
        type Program = Bind<IdK, Pure<IdK, Unit>, LConst<Pure<IdK, Pair<Unit, Unit>>>>;
        type Result = Evaluate<RunId<Program>>;
        assert_type_eq_all!(Result, Pair<Unit, Unit>);
    }
}
