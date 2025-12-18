use std::marker::PhantomData;

use typelude::{Evaluate, lambda::*};

// =========================================================================
// RPN Refactoring Draft: BinaryOp Abstraction
// =========================================================================

// -------------------------------------------------------------------------
// 1. 抽象化レイヤー (The Abstraction Layer)
// -------------------------------------------------------------------------

/// 2項演算ロジックを表すトレイト。
/// スタック操作の詳細は知らず、単に「AとBを受け取って結果を返す」ことだけを定義する。
trait BinaryLogic {
    /// 実際の計算を行う型レベル関数 (例: LAdd)
    /// Apply2<F, A, B> 可能なもの
    type Op;
}

/// 2項演算を行うスタックマシンの命令。
/// これが「Lambdaトレイトで吸収」する部分に相当します。
/// 「BinaryLogic」を満たす型 T に対して、自動的に Apply<Stack> を実装します。
struct OpBinary<T: BinaryLogic>(PhantomData<T>);

// ヘルパー: カリー化適用
type Apply2<F, A, B> = <<F as Apply<A>>::Output as Apply<B>>::Output;

// ヘルパー: スタック操作の結果型
type PopResult<S> = <ActionPop as Apply<S>>::Output;
trait LPairTerm {
    type Fst;
    type Snd;
}
impl<A, B> LPairTerm for LPair<A, B> {
    type Fst = A;
    type Snd = B;
}
type PopVal<S> = <PopResult<S> as LPairTerm>::Fst;
type PopStack<S> = <PopResult<S> as LPairTerm>::Snd;

// アクション: Pop (これはプリミティブなのでそのまま)
struct ActionPop;
impl<S> Apply<S> for ActionPop
where
    S: LList,
    LHeadOr<S, LZero>: Lambda,
    LTailOr<S, LNil>: Lambda,
{
    type Output =
        LPair<<LHeadOr<S, LZero> as Lambda>::Output, <LTailOr<S, LNil> as Lambda>::Output>;
}

// -------------------------------------------------------------------------
// 2. 複雑さの封じ込め (Encapsulation)
// -------------------------------------------------------------------------

// ここが重要です。
// 「2回Popして、Opを適用して、Pushする」という複雑な制約の連鎖は、
// この blanket implementation に**1回だけ**書きます。
impl<Logic, S> Apply<S> for OpBinary<Logic>
where
    Logic: BinaryLogic,
    S: LList,
    // 1. Pop B
    ActionPop: Apply<S>,
    PopResult<S>: LPairTerm,
    // 2. Pop A
    ActionPop: Apply<PopStack<S>>,
    PopResult<PopStack<S>>: LPairTerm,
    // 3. Apply Op (A B)
    Logic::Op: Apply<PopVal<PopStack<S>>>,
    <Logic::Op as Apply<PopVal<PopStack<S>>>>::Output: Apply<PopVal<S>>,
    // 4. Result -> Term conversion (optional but good for robustness)
    // Apply2<Logic::Op, PopVal<PopStack<S>>, PopVal<S>>: LTerm, // Removed: Not strictly necessary for LCons
    // 5. Result Pushable
    // スタックに積む結果となる型
    PopStack<PopStack<S>>: LList, // Added: Mandatory for LCons
    LCons<Apply2<Logic::Op, PopVal<PopStack<S>>, PopVal<S>>, PopStack<PopStack<S>>>: LList,
{
    // 戻り値: ((), NewStack)
    // NewStack = Cons<Op<A, B>, RestStack>
    type Output = LPair<
        LNil,
        LCons<Apply2<Logic::Op, PopVal<PopStack<S>>, PopVal<S>>, PopStack<PopStack<S>>>,
    >;
}

// -------------------------------------------------------------------------
// 3. ユーザーコード (User Code)
// -------------------------------------------------------------------------
// ユーザーはこれだけ書けばOKになります。
// Applyの制約地獄からは解放されます。

// Add定義
struct LogicAdd;
impl BinaryLogic for LogicAdd {
    type Op = LAdd;
}
/// RPN命令としての Add
type OpAdd = OpBinary<LogicAdd>;

// Mul定義
struct LogicMul;
impl BinaryLogic for LogicMul {
    type Op = LMul;
}
/// RPN命令としての Mul
type OpMul = OpBinary<LogicMul>;

// Push定義 (これも専用の抽象化を作れるが、単純なので割愛)
struct OpPush<N>(PhantomData<N>);
impl<N, S> Apply<S> for OpPush<N>
where
    S: LList,
{
    type Output = LPair<LNil, LCons<N, S>>;
}

// -------------------------------------------------------------------------
// 4. テスト (Verification)
// -------------------------------------------------------------------------

// ランナー
struct RunProgram<Ops>(PhantomData<Ops>);
impl<S> Apply<S> for RunProgram<LNil> {
    type Output = S;
}
impl<Op, Rest, S> Apply<S> for RunProgram<LCons<Op, Rest>>
where
    Op: Apply<S>,
    Op::Output: LPairTerm,
    Rest: LList, // Added: Mandatory
    RunProgram<Rest>: Apply<<Op::Output as LPairTerm>::Snd>,
{
    type Output = <RunProgram<Rest> as Apply<<Op::Output as LPairTerm>::Snd>>::Output;
}

#[test]
fn test_refactored_rpn() {
    use static_assertions::assert_type_eq_all;

    // 2 3 +
    type Prog = LCons<
        OpPush<LSucc<LSucc<LZero>>>,
        LCons<OpPush<LSucc<LSucc<LSucc<LZero>>>>, LCons<OpAdd, LNil>>,
    >;

    type ResultStack = <RunProgram<Prog> as Apply<LNil>>::Output;

    // Normalize Check
    type TopExpr = <LHeadOr<ResultStack, LZero> as Lambda>::Output;
    type Normalized = <<TopExpr as Apply<LSuccGen>>::Output as Apply<LZero>>::Output;

    assert_type_eq_all!(Evaluate<Normalized>, Evaluate<LSucc<LSucc<LSucc<LSucc<LSucc<LZero>>>>>>);
}
