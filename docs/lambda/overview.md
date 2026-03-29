# typelude-std/src/lambda — ラムダ計算実装概要

`lambda` モジュールは、Rustの型システム上にラムダ計算の意味論を実装したものである。
`Eval` トレイトによるβ簡約を軸に、Church符号化・コンビネータ・モナドまでを型レベルで構成する。

---

## 目次

1. [基盤：適用の意味論](#1-基盤適用の意味論)
2. [Church符号化](#2-church符号化)
   - [真偽値](#21-church真偽値-boolrs)
   - [数と算術](#22-church数とペア算術-numeralrs-pairrs)
   - [対](#23-church対-pairrs)
   - [制御フロー](#24-制御フロー-controlrs)
3. [SKIコンビネータ計算](#3-skiコンビネータ計算-skirs)
4. [不動点コンビネータ](#4-不動点コンビネータ-fixrs)
5. [Churchリスト](#5-churchリスト-listrs)
6. [サンク（遅延評価）](#6-サンク遅延評価-thunkrs)
7. [カリー化変換](#7-カリー化変換-curryrs)
8. [モナド](#8-モナド-monads)
9. [型レベル定理証明](#9-型レベル定理証明-proofrs)

---

## 1. 基盤：適用の意味論

**ファイル**: `traits.rs`, `mod.rs`

| 要素 | 説明 |
|---|---|
| `LApp<F, A>` | 関数適用 `F A` を表す型レベルAST |
| `LTerm` | 全ラムダ項のマーカートレイト |
| `LBool`, `LNat`, `LList` | Church符号化された値の意味論クラス |
| `LBind<F>` | モナドの `>>=`（bind）操作の型レベル表現 |

**評価戦略**：`Eval` + `Evaluate<T>` による Call-by-Value。
引数を先に正規化（`Evaluate<Arg>`）してから適用するため、β簡約は型推論として実行される。

```
LApp<F, A> : Eval  →  type Output = ...（β簡約の結果）
```

---

## 2. Church符号化

### 2.1 Church真偽値 (`bool.rs`)

Church真偽値は「2引数関数のどちらを選ぶか」として定義される。

| 構造 | ラムダ定義 | 意味 |
|---|---|---|
| `LTrue` | `λt f. t` | 真：第1引数を選択 |
| `LFalse` | `λt f. f` | 偽：第2引数を選択 |
| `LTrue1<T>`, `LFalse1<T>` | カリー化の中間状態 | 部分適用の正規形 |
| `LIf` / `LIf1<P>` / `LIf2<P,T>` | `λp t e. p t e` | 条件分岐（3引数カリー化） |
| `LPureIf<P,T,E>` | `((P T) E)` の展開済み型エイリアス | 正規化済み条件分岐 |

```
LTrue  A B → A
LFalse A B → B
LIf LTrue  T E → T
LIf LFalse T E → E
```

### 2.2 Church数とペア算術 (`numeral.rs`, `pair.rs`)

Church数は「関数 `f` を `n` 回 `x` に適用する」高階関数として定義される。

| 構造 | ラムダ定義 | 意味 |
|---|---|---|
| `LZero` | `λf x. x` | 0：`f` を0回適用 |
| `LSucc<N>` | `λf x. f (N f x)` | 後継子：`f` を1回多く適用 |
| `LSuccGen` | `λn. Succ<n>` | 後継子生成器 |
| `LAdd` | `λm n f x. m f (n f x)` | 加算 |
| `LMul` | `λm n f x. m (n f) x` | 乗算 |
| `LExp` | `λm n f x. (n m) f x` | 冪乗 |
| `LPred` | `λn. Fst (n PredStep (Pair 0 0))` | 前者関数（Kleene's predecessor）|
| `LPredStep` | `λp. Pair (Snd p) (Succ (Snd p))` | Pred計算のステップ関数 |
| `LSub` | `λm n. (n Pred) m` | 飽和減算 |

各算術演算子は4引数カリー化で実装されており、中間状態 `L*1`, `L*2`, `L*3` が部分適用の段階を管理する。

**Kleene's predecessor** について：`LPred` はペアエンコーディングを利用する標準的手法で実装されている。
`N PredStep (Pair 0 0)` は `(0,0) → (0,1) → (1,2) → ... → (n-1, n)` とシフトし、最後に `Fst` で前者を取り出す。

### 2.3 Church対 (`pair.rs`)

| 構造 | ラムダ定義 | 意味 |
|---|---|---|
| `LPair` | `λx y. λf. f x y` | 対の構成子 |
| `LPair2<X,Y>` | 完全適用済みの正規形 | 対の値 |
| `LFst` | `λp. p True` | 第1要素射影 |
| `LSnd` | `λp. p False` | 第2要素射影 |

`LFst`/`LSnd` はChurch真偽値をセレクタとして利用している（`True = K`、`False = KI` との対応）。

### 2.4 制御フロー (`control.rs`)

| 構造 | 意味論 |
|---|---|
| `LWhile` | `while pred body state = if (pred state) then while pred body (body state) else state` |
| `LWhileHelper` | `LTrue`/`LFalse` でのトレイト dispatch による分岐 |
| `LFor` | Churchリストへの反復（宣言のみ、未実装） |

`LWhile` はβ簡約の再帰として型チェッカが展開する。停止条件（`pred state = LFalse`）が満たされるまで型推論が再帰的にunrollされる。

---

## 3. SKIコンビネータ計算 (`ski.rs`)

SKIコンビネータはラムダ計算のチューリング完全な基底を成す。

| 構造 | ラムダ定義 | 意味 |
|---|---|---|
| `I` | `λx. x` | 恒等子 |
| `K` | `λx y. x` | 定数関数（第1投影）|
| `S` | `λx y z. (x z)(y z)` | 置換子（分配適用）|
| `K1<X>` | `K X` の部分適用 | `λy. X` |
| `S1<X>`, `S2<X,Y>` | `S X`、`S X Y` の部分適用 | 中間正規形 |

**注目する性質**：
- `SKK x → x`（`I` の SKI 表現）が型等価として成立
- `K = Church True`、`KI = Church False` の同型をテストで確認
- `S` の適用は `Z: Clone` 制約が必要（同一型引数を2箇所に渡すため）

---

## 4. 不動点コンビネータ (`fix.rs`)

| 構造 | 意味論 |
|---|---|
| `LFix<F>` | 不動点コンビネータ |
| `LApp<LFix<F>, X>` | `Fix F X → (F (Fix F)) X` |

**簡約規則**：
```
Fix F X  ⟶  (F (Fix F)) X
```

これはYコンビネータ相当の型レベル実装である。Rustの型チェッカは有限ステップで停止する再帰のみを受理するため、型レベル無限ループは型エラーとなる。

`LThunk` と組み合わせることで、ステップ単位の遅延評価（step-by-step unrolling）が可能になる（`fix.rs` のテスト参照）。

---

## 5. Churchリスト (`list.rs`)

リストはChurch符号化により「畳み込み関数」として定義される。

| 構造 | ラムダ定義 | 意味 |
|---|---|---|
| `LNil` | `λc n. n` | 空リスト |
| `LCons` | `λh t. λc n. c h t` | リスト構成子 |
| `LCons2<H,T>` | 完全適用済みの正規形 | `h :: t` の値 |
| `LHeadOr` | `λl d. l K d` | 先頭要素（デフォルト付き）|
| `LTailOr` | `λl d. l False d` | 尾部（デフォルト付き）|
| `LIsEmpty` | `λl. l (λh t. False) True` | 空リスト判定 |
| `LFoldr` | `λf z l. l (λh t. f h (foldr f z t)) z` | 右畳み込み |
| `LPureUncons<L,OnCons,OnNil>` | `L OnCons OnNil` | case分解のエイリアス |

`LFoldr` の再帰はβ展開として型推論時に展開されるため、有限リストのみ停止が保証される。

---

## 6. サンク（遅延評価）(`thunk.rs`)

| 構造 | 意味論 |
|---|---|
| `LThunk<F, Arg>` | 中断された計算 `F Arg`（値としてキャプチャ）|
| `LForce` | 評価を引き起こすトークン |
| `LApp<LThunk<F,Arg>, LForce>` | `→ Evaluate<LApp<F, Arg>>` |

```
Thunk<F, Arg> Force  →  F Arg
```

意味論的には `λ_. F Arg`（Unit引数の関数）に相当する。`LFix` と組み合わせることで、型チェッカの強制的な展開を制御するステップ遅延として機能する。

---

## 7. カリー化変換 (`curry.rs`)

| 構造 | 意味論 |
|---|---|
| `LCurry<F>` | タプル引数 `(A, B) → C` をカリー形式 `A → B → C` に変換 |
| `LCurry1<F,A>` | 第1引数を部分適用した中間形 |
| `LUncurry<F>` | カリー形式 `A → B → C` をタプル引数形式 `LPair2<A,B> → C` に変換 |

```
Curry<F> A B        →  F (Pair A B)
Uncurry<F> (Pair A B)  →  (F A) B
```

`LPair2<A,B>` をタプルの型レベル表現として使用している。`Curry ∘ Uncurry` の往還が型等価として成立することをテストで確認。

---

## 8. モナド (`monads/`)

`LBind<F>` トレイトが `>>=` を統一的に表現する。各モナドはこのトレイトを実装することで計算効果を型レベルで合成する。

### 8.1 Identity モナド (`identity.rs`)

最も単純なモナド。値のラッパーとして機能する。

```
LId<T> >>= F  →  F T
```

### 8.2 Either モナド (`either.rs`)

エラー処理のための直和型。`Left` がエラー、`Right` が成功を表す。

| 操作 | 規則 |
|---|---|
| `LLeft<L>` | エラー値（Church符号化: `λl r. l L`）|
| `LRight<R>` | 成功値（Church符号化: `λl r. r R`）|
| `LLeft<L> >>= k` | `→ LLeft<L>`（エラーを伝播、短絡）|
| `LRight<R> >>= k` | `→ k R`（継続を適用）|

Church真偽値と同型の2分岐構造。パターンマッチはトレイト実装の分岐として表現される。

### 8.3 State モナド (`state.rs`)

状態を明示的に渡す計算モデル。

| 操作 | 規則 |
|---|---|
| `LState<F>` | 関数 `S → (A, S)` のラッパー |
| `LApp<LState<F>, S>` | `→ F S`（状態を渡して実行）|
| `LState<F> >>= k` | `→ LState<LBindState<F,k>>`（状態関数を合成）|
| `LReturn<A>` | `λs. Pair(A, s)`（値を状態付きで返す）|
| `LGet` | `λs. Pair(s, s)`（状態を読む）|
| `LPut<NewS>` | `λold. Pair((), NewS)`（状態を書く）|

```
BindState<F, K> s  →  let (a, s') = F s in K a s'
```

`LFst`/`LSnd` によって対から値と新状態を取り出す。

### 8.4 CPSモナド (`cps.rs`)

継続渡しスタイルによる計算の合成。

| 操作 | 規則 |
|---|---|
| `LCont<F>` | `(A → R) → R` 型の計算（`F` が内部関数）|
| `LRunCont<LCont<F>, K>` | `→ F K`（継続 `K` を渡して実行）|
| `LPureF<A>` | `λk. k A`（値 `A` を継続に渡す）|
| `LCont<F> >>= G` | `→ LCont<LBindF<F,G>>`（継続合成）|

**bind の CPS変換規則**：
```
(m >>= f) k  =  m (λa. f a k)
```

型レベルでは `LBindF<F,G>` と `LBindK<G,K>` の2層構造で表現される。

---

## 9. 型レベル定理証明 (`proof.rs`)

Curry-Howard対応により「命題 = 型、証明 = 値」として定理を型レベルで表現する。

| 構造 | 対応する論理規則 |
|---|---|
| `LRefl<A>` | 反射性：`refl : A = A` |
| `LTypeEq<B>` | 命題 `A = B` のマーカートレイト |
| `LSym<Proof>` | 対称性：`A = B → B = A` |
| `LTrans<P1, P2>` | 推移性：`A = B, B = C → A = C` |
| `LCong<F, Proof>` | 合同性：`A = B → F A = F B`（HKT制約により構造のみ定義）|

`LSym` と `LTrans` は `LRefl` 上で `Eval` として `LRefl` に簡約される。
`LCong` はRustのHigh-Kinded Types不足により完全実装は困難なため、型の宣言のみ存在する。
