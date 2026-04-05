# v2 型レベル高階適用 API パターン設計

## 検討ドキュメント

2026-04-06

---

## 背景

Rustの型レベルプログラミングで高階関数を実現する際、関数適用をどのように型システムに表現するかについて、複数のAPI設計パターンが考えられます。

この文書は、rustc の trait 解決・投影正規化の観点から、高階適用を実現しうる22の設計パターンを列挙し、6つの評価軸で比較したものです。

---

## 前提

### 問題設定

While や Map のような高階操作では、演算子（関数）Fn と引数 Arg をジェネリクスで合成する必要があります。

```rust
// 素朴な試み（失敗）
type MapResult<Fn, List> = Map<Fn, List>;  // OK
type MapResult2<X> = Map<X, X>;             // NG: Generics<Generics> 不可
```

Rust の型システムでは直接的に Generics<Generics> を書けないため、**別の抽象を経由する必要があります**。

### 評価軸

以下6つの観点から各パターンを評価します：

1. **直感性**
   ロジック定義API利用者にとってのメンタルモデルのわかりやすさ（impl の書き味）

2. **rustc都合**
   高階化における衝突の少なさ、trait解決のしやすさ

3. **AST/API互換**
   AST的な記述/デバッグに都合がいいAPIの提供、変換の摩擦の少なさ

4. **境界簡便**
   副産物としてのトレイト境界記述の簡便さ

5. **一般化強制**
   共通API・trait解決モデルの強制力

6. **汎用拡張**
   ライブラリ化時の利用者による拡張性（引数数など含む）

評価値: 高 ≥ 中 ≥ 低 (あるいは 中高 等の中間値)

---

## 22 パターン一覧

| No  | パターン名                         | 概要                                    |
| --- | ---------------------------------- | --------------------------------------- |
| 1   | Self=Op, Apply<Arg> for Op         | Op型の trait に Apply<Arg> を実装       |
| 2   | Self=Arg, Apply<Op> for Arg        | Arg型の trait に Apply<Op> を実装       |
| 3   | Self=(Op, Arg), Apply for (Op,Arg) | ペア型の trait に Apply を実装          |
| 4   | Self=(), Apply<Op, Arg> for ()     | グローバル型に Apply<Op, Arg> を実装    |
| 5   | Pair型キャリア                     | Pair<Op, Arg> 構造体を所有者にする      |
| 6   | Eval<EApp<Op,Arg>> for ()          | EApp の評価を () の trait に実装        |
| 7   | Expr自己評価                       | Eval trait を Expr 型自身に持たせる     |
| 8   | Op側GAT (TypeFn::Apply<Arg>)       | Op に GAT Apply<Arg> を定義             |
| 9   | Arg側GAT (AppliedBy::Apply<Op>)    | Arg に GAT Apply<Op> を定義             |
| 10  | Callable層分離                     | Callable + Apply<Arg> で2層化           |
| 11  | Arity別トレイト                    | Apply1, Apply2, Apply3, ... で固定arity |
| 12  | 引数パック                         | Apply<(A, B, ...)> で多引数を1族        |
| 13  | カリー化キャプチャ型               | Op → Op<A> → Op<A,B> と段階生成         |
| 14  | Defunctionalization記号            | 記号型とその意味をトレイトで定義        |
| 15  | AST Lower化                        | Ast → Core への変換層を明示             |
| 16  | 二段評価                           | EvalAst + EvalCore で分離               |
| 17  | Small-step正規化                   | Step + Normalize の再帰合成             |
| 18  | 関係述語                           | ReducesTo<Op, Arg> で関係定義           |
| 19  | CPS継続                            | ApplyK<Arg, Cont> で継続を型参数に      |
| 20  | 文脈付き評価                       | ApplyIn<Ctx, Arg>, EvalIn<Ctx, Expr>    |
| 21  | Kindタグ付き                       | ApplyKind<K, Arg> で演算空間分離        |
| 22  | ラムダAST+環境                     | Var/Lam/App + Env で λ計算直接実装      |

---

## 6観点での比較表

| No  | パターン              | 直感性 | rustc都合 | AST/API | 境界簡便 | 一般化強制 | 汎用拡張 | 簡潔評注                     |
| --- | --------------------- | ------ | --------- | ------- | -------- | ---------- | -------- | ---------------------------- |
| 1   | Apply<Arg> for Op     | 高     | 高        | 高      | 高       | 高         | 高       | 関数適用をそのままtraitへ    |
| 2   | Apply<Op> for Arg     | 中低   | 中        | 中低    | 中       | 中         | 中       | 演算子中心の認知とズレ       |
| 3   | Apply for (Op,Arg)    | 中     | 中        | 中      | 中       | 中低       | 中       | 実装単位が組になる           |
| 4   | Apply<Op, Arg> for () | 中     | 中高      | 中      | 中高     | 高         | 中低     | 大域関係として明示的         |
| 5   | Pair<Op,Arg>キャリア  | 中高   | 中高      | 高      | 中       | 高         | 高       | キャリア型で関係を明示       |
| 6   | Eval<EApp> for ()     | 中     | 中        | 中高    | 中       | 高         | 中低     | 評価入口は一元化             |
| 7   | Expr自己評価          | 高     | 中高      | 高      | 高       | 高         | 高       | AST中心モデルと高階を同居    |
| 8   | Op側GAT               | 中高   | 中高      | 中高    | 高       | 高         | 高       | 演算族をOpへ集約             |
| 9   | Arg側GAT              | 中低   | 中        | 中      | 中       | 中         | 中       | 責務がArg側へ寄る            |
| 10  | Callable+Apply        | 高     | 高        | 中高    | 高       | 高         | 高       | 呼び出し可能性と適用を分離   |
| 11  | Arity別               | 高     | 高        | 中高    | 高       | 中         | 中       | 固定arity 毎の面が増える     |
| 12  | 引数パック            | 中高   | 中高      | 高      | 高       | 高         | 高       | 入口1つで多引数に対応        |
| 13  | カリー化キャプチャ    | 中     | 中        | 高      | 中       | 高         | 高       | 部分適用を自然に表現         |
| 14  | Defeunctionalization  | 中高   | 高        | 中高    | 高       | 高         | 高       | 記号と意味を分離             |
| 15  | AST Lower             | 高     | 中高      | 高      | 中高     | 高         | 高       | 記述層と計算層を分離         |
| 16  | 二段評価              | 中高   | 中        | 高      | 中       | 高         | 高       | フェーズ分離が明示的         |
| 17  | Small-step            | 中低   | 低        | 高      | 低       | 高         | 中高     | 意味論は明瞭だが負荷高       |
| 18  | ReducesTo関係述語     | 中     | 中高      | 中      | 中       | 中高       | 中高     | 関係定義として扱いやすい     |
| 19  | CPS継続               | 低     | 中低      | 中      | 低       | 中高       | 中       | 制御表現は強いが認知負荷高   |
| 20  | 文脈付き              | 中     | 中        | 中高    | 中       | 高         | 高       | 評価戦略を型パラメータで制御 |
| 21  | Kindタグ付き          | 中     | 中高      | 中高    | 中高     | 高         | 高       | 演算空間の分離に強い         |
| 22  | ラムダAST+環境        | 中     | 低        | 高      | 中低     | 高         | 高       | λ計算に忠実だが評価系重い    |

---

## グループ分類

### グループ分け基準

各パターンを設計の志向で3群に分類します：

#### A. Op + API互換群 (11パターン)

**番号:** 1, 4, 5, 8, 10, 11, 12, 13, 14, 20, 21

**志向:** Op型中心に Apply トレイトを構築。延長性と API 統一性を重視。

**特性:**

- 演算子オブジェクトとしてのOp実装を単一の族で統治
- 呼び出し側からは Op: Apply<Arg> という一貫した境界
- 拡張時は Op の新規実装だけで完結しやすい
- trait 解決が比較的素直

**代表例:**

- No.1: Self=Op 形式（最もシンプル、関数型言語的）
- No.12: 引数パック（多引数を統一入口で扱う）
- No.13: カリー化キャプチャ（部分適用を自然に表現）

---

#### B. AST + 高階化互換群 (6パターン)

**番号:** 6, 7, 15, 16, 17, 22

**志向:** ASTの記述性を維持しながら高階化を実現。意味論の明確化を重視。

**特性:**

- ユーザー向けは AST 記法（And<A,B>, If<Cond,Then,Else> etc.）
- 内部で計算層へ変換・評価
- デバッグとトレーサビリティに強い
- trait 層は増えやすいが責務が明確

**代表例:**

- No.7: Expr自己評価（AST型が Eval trait 実装、最も直感的）
- No.15: AST Lower化（記述層と計算層を明確分離）
- No.16: 二段評価（ASTの評価 → Core の評価、という段階が見える）

---

#### C. その他 (5パターン)

**番号:** 2, 3, 9, 18, 19

**志向:** 上記2群のいずれとも親和性が低いか、特殊用途向け。

**特性:**

- 2 (Self=Arg): データ型が主語になり、演算子中心の直感と逆向き
- 3 (ペア型): 実装単位が組になり粒度管理が微細
- 9 (Arg側GAT): Arg側に責務を寄せるため、関数型パラダイムと逆向き
- 18 (関係述語): 関係定義で扱いやすいが関数感は薄い
- 19 (CPS): 制御表現に強いが通常演算には認知負荷が高い

---

## 群別サマリ

### Op + API互換群

**主張:**
"Op型に Apply トレイトを実装することで、高階適用を統一的に表現。拡張性と API 習熟の効率を優先。"

**適性:**

- ライブラリAPIとしての安定性 ◎
- 実装者の負荷 ◎（新規Op族は define_op!macro で自動生成可能）
- デバッグ ○（AST層がないため、型エラーは計算層に直結）
- 試験時の検証API ◎（引数や結果を型で直接指定）

**リスク:**

- ユーザーが見るトレイト境界が計算寄りになり、高階の複雑性が隠蔽されにくい
- 中間型が多数生成される可能性

---

### AST + 高階化互換群

**主張:**
"ユーザーAPI は AST 記法で読みやすく。内部で変換・評価。意味論の透明性と保守性を優先。"

**適性:**

- ユーザーメンタルモデル ◎（And, If 等の構文に見える）
- デバッグ ◎（AST から Lowering, Eval の流れが追跡可能）
- 試験時の検証API ◎（Reify / Debug による値化が容易）
- 学習曲線 ◎（型レベル計算を「式の構築と評価」として理解しやすい）

**リスク:**

- 変換層が増えることで trait resolution の深さが増す
- AST→Core 正規化の過程で型がひとつ増える

---

### その他群

**評価:**

- 特定ユースケースには強い（CPS の制御フロー表現など）
- 一般汎用ライブラリには適さない傾向
- パターン毎に特殊な学習コストが生じやすい
- v2の正規設計には組み込みにくい

---

## パターン別コード例

以下は各パターンの具体的な実装例とAST/利用例です。
各例は Add<A, B> = A + B という基本操作と Map<F, List> という高階操作を示しています。

### グループ A: Op + API互換群

---

#### パターン#1: Self=Op, Apply<Arg> for Op

**特徴:** Op型自身に Apply<Arg> trait を実装。最もシンプルで関数型言語的。

**Trait定義:**

```rust
pub trait Eval { type Output; }
pub key trait Apply<Arg> { type Output: ?Sized; }

// Op実装例: Add
pub struct Add<A, B>(PhantomData<(A, B)>);

impl<A: Eval, B: Eval> Apply<B> for Add<A>
{
    type Output = /* A.Output + B.Output */;
}

impl<A: Eval, B: Eval> Eval for Add<A, B>
where
    Add<A>: Apply<B>,
{
    type Output = <Add<A> as Apply<B>>::Output;
}
```

**利用例 (AST記法):**

```rust
type Result = Evaluate<Add<ELit<U3>, ELit<U5>>>; // U8

// 高階: Map<Add<X>, List> は、各要素にAdd<X>を適用
type Mapped = Evaluate<EMap<Add<ELit<U1>>, [U1, U2, U3]>>;
// → [U2, U3, U4]
```

**簡潔評注:** 最も直感的で拡張しやすい。トレイト境界は `where X: Apply<Y>` の一言で済みます。

---

#### パターン#4: Self=(), Apply<Op, Arg> for ()

**特徴:** グローバル型 () に Apply<Op, Arg> を実装。演算の大域関係として明示的。

**Trait定義:**

```rust
pub trait Apply<Op, Arg> { type Output; }

// Op側は単なるマーカー型
pub struct Add;
pub struct Mul;

impl<A: Eval, B: Eval> Apply<Add, (Tuple<A>, Tuple<B>)> for ()
{
    type Output = /* A.Output + B.Output */;
}

impl<Op, Arg> Eval for EApp<Op, Arg>
where
    (): Apply<Op, Arg>,
{
    type Output = <() as Apply<Op, Arg>>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<EApp<Add, (Tuple<U3>, Tuple<U5>)>>; // U8

// 高階の場合、関数をOp型でキャプチャ
struct MapOp<F>(PhantomData<F>);
impl<Op, List> Apply<MapOp<Op>, List> for ()
where
    /* List 要素に Op を Apply */
{
    type Output = /* 結果 */;
}
```

**簡潔評注:** 演算を大域述語として扱う。trait 解決が明示的。拡張時は新規 impl を追加するだけ。

---

#### パターン#5: Pair型キャリア

**特徴:** Pair<Op, Arg> 型をキャリアにして、その上に Eval を実装。

**Trait定義:**

```rust
pub struct Pair<Op, Arg>(PhantomData<(Op, Arg)>);

impl<Op: Eval, Arg: Eval> Eval for Pair<Op, Arg>
where
    <Op as Eval>::Output: Apply<<Arg as Eval>::Output>,
{
    type Output = <<Op as Eval>::Output as Apply<<Arg as Eval>::Output>>::Output;
}

pub struct Add;
impl Eval for Add { type Output = AddFn; }

pub struct AddFn;
impl<X> Apply<X> for AddFn { /* ... */ }
```

**利用例:**

```rust
type R = Evaluate<Pair<Add, ELit<U3>>>;  // AddFn (部分適用)
type R2 = Evaluate<Pair<Pair<Add, U3>, U5>>; // U8

// 高階: Map も Pair で構築
type MapAdd = Pair<Map, Add>;
```

**簡潔評注:** キャリア型が明示的。関数と引数の関係性が型構造に表れます。

---

#### パターン#8: Op側 GAT (TypeFn::Apply<Arg>)

**特徴:** Op 型に GAT Apply<Arg> を定義。arity別の族を型パラメータで管理。

**Trait定義:**

```rust
pub trait TypeFn { type Apply<Arg>; }

pub struct Add;
impl TypeFn for Add { type Apply<X> = Add<X>; }

pub struct Add<A>(PhantomData<A>);
impl<A> TypeFn for Add<A> { type Apply<B> = /* 結果型 */; }

impl<A: Eval, B: Eval> Eval for Add<A, B> {
    type Output = /* A.out + B.out */;
}
```

**利用例:**

```rust
type Part1 = <Add as TypeFn>::Apply<ELit<U3>>;  // Add<ELit<U3>>
type Result = Evaluate<Add<ELit<U3>, ELit<U5>>>;

// 高階: Map<F, List> で、F は TypeFn
struct Map<F, L>(PhantomData<(F, L)>);
impl<F: TypeFn, L> Eval for Map<F, L>
where
    L: ListOf,
    /* 要素型に <F as TypeFn>::Apply を適用 */
{
    type Output = /* マップ結果 */;
}
```

**簡潔評注:** GAT を活用して演算族を型に集約。表現力が高く、trait resolution も安定します。

---

#### パターン#10: Callable層分離

**特徴:** Callable と Apply<Arg> で2層化。呼び出し可能性を明示的に分離。

**Trait定義:**

```rust
pub trait Callable { type Fn; }
pub trait Apply<Arg> { type Output; }

pub struct Add<A>(PhantomData<A>);

impl<A> Callable for Add {
    type Fn = Add<A>;  // 固定化されている場合も、明示的な層を用意
}

impl<A: Eval, B: Eval> Apply<B> for Add<A> {
    type Output = /* A + B */;
}

impl<A: Eval, B: Eval> Eval for Add<A, B>
where
    Add<A>: Callable,
    <Add<A> as Callable>::Fn: Apply<B>,
{
    type Output = <<Add<A> as Callable>::Fn as Apply<B>>::Output;
}
```

**利用例:**

```rust
type AddU3 = <Add as Callable>::Fn;  // Add<U3> (呼び出し可能形)
type Result = Evaluate<Add<U3, U5>>;

// 高階: Callable に Apply を束ねることで高階を実現
```

**簡潔評注:** 呼び出し可能性を型として表現。デバッグ時に「どのレイヤーか」が明確。

---

#### パターン#11: Arity別トレイト

**特徴:** Apply1, Apply2, Apply3, ... で固定 arity ごとにトレイト分離。

**Trait定義:**

```rust
pub trait Apply1<A> { type Output; }
pub trait Apply2<A, B> { type Output; }
pub trait Apply3<A, B, C> { type Output; }

pub struct Add;

impl<A: Eval, B: Eval> Apply2<A, B> for Add {
    type Output = /* A + B */;
}

impl<A: Eval, B: Eval> Eval for EApp2<Add, A, B>
{
    type Output = <Add as Apply2<A, B>>::Output;
}

pub struct Map;
impl<F: ???, L: ???> Apply2<F, L> for Map {
    type Output = /* マップ結果 */;
}
```

**利用例:**

```rust
type R = Evaluate<EApp2<Add, ELit<U3>, ELit<U5>>>; // U8
type M = Evaluate<EApp2<Map, /* F */, [U1, U2]>>;

// 高階: F そのものが Apply2 を実装していれば、Map で再利用可能
```

**簡潔評注:** arity が固定化されるため型エラーが明確。ただしtraitが乗算的に増えます。

---

#### パターン#12: 引数パック

**特徴:** Apply<(A, B, ...)> で多引数を1つの trait に統一。

**Trait定義:**

```rust
pub trait Apply<Args> { type Output; }

pub struct Add;

impl<A: Eval, B: Eval> Apply<(A, B)> for Add {
    type Output = /* A + B */;
}

impl<A: Eval, B: Eval> Eval for EApp<Add, (A, B)>
where
    Add: Apply<(A, B)>,
{
    type Output = <Add as Apply<(A, B)>>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<EApp<Add, (ELit<U3>, ELit<U5>)>>; // U8

// 高階: タプルをそのまま合成
type MapAdd = Map<Add, [U1, U2, U3]>;
// Map は (Add, elem) ペアを自動生成して Apply<(Add, elem)> を呼び出し
```

**簡潔評注:** 入口が1つで多引数対応。impl 数が少ないので trait resolution が効率的。

---

#### パターン#13: カリー化キャプチャ型

**特徴:** Op → Op<A> → Op<A,B> と段階的に型を生成。部分適用を自然に表現。

**Trait定義:**

```rust
pub trait Apply<A> { type Output; }

pub struct Add;

impl<A: Eval> Apply<A> for Add {
    type Output = Add<A>;  // 部分適用
}

pub struct Add<A>(PhantomData<A>);

impl<A: Eval, B: Eval> Apply<B> for Add<A> {
    type Output = /* A + B */;
}

impl<A: Eval, B: Eval> Eval for Add<A, B>
where
    Add: Apply<A>,
    <Add as Apply<A>>::Output: Apply<B>,
{
    type Output = <<Add as Apply<A>>::Output as Apply<B>>::Output;
}
```

**利用例:**

```rust
type AddU3 = <Add as Apply<ELit<U3>>>::Output;  // Add<ELit<U3>>
type Result = <AddU3 as Apply<ELit<U5>>>::Output;  // U8

// 高階: 部分適用を自然に表現
type MapAddOne = Map<Add, ListU1>;
// 内部的に、各要素xに <Add as Apply<x>>::Output として適用
```

**簡潔評注:** 関数型言語のカリー化をそのまま型に実装。部分適用が言語構造に組み込まれます。

---

#### パターン#14: Defunctionalization記号

**特徴:** 記号型と意味をトレイトで分離。抽象化層を明示的に。

**Trait定義:**

```rust
pub trait Def { type Meaning; }

pub struct AddSym;
impl Def for AddSym { type Meaning = Add; }

pub struct Add;
impl<A: Eval, B: Eval> Apply<(A, B)> for Add {
    type Output = /* A + B */;
}

// 利用側は記号を操作
impl Eval for EApp<AddSym, (A, B)>
where
    <AddSym as Def>::Meaning: Apply<(A, B)>,
{
    type Output = <<AddSym as Def>::Meaning as Apply<(A, B)>>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<EApp<AddSym, (U3, U5)>>; // U8

// 高階: 記号をそのまま関数として扱える
type MapAddSym = Map<AddSym, [U1, U2]>;
```

**簡潔評注:** 記号と意味を分離することで、中間表現のバリデーション・変換が容易。

---

#### パターン#20: 文脈付き評価

**特徴:** ApplyIn<Ctx, Arg>, EvalIn<Ctx, Expr> で評価戦略を型パラメータで制御。

**Trait定義:**

```rust
pub trait Context { type Mode; }

pub struct StrictCtx;
impl Context for StrictCtx { type Mode = Strict; }

pub trait ApplyIn<Ctx, Arg> { type Output; }

pub struct Add;

impl<Ctx: Context, A: EvalIn<Ctx>, B: EvalIn<Ctx>>
    ApplyIn<Ctx, (A, B)> for Add
where
    Ctx::Mode: ???,
{
    type Output = /* A.out + B.out */;
}

impl<Ctx, A, B> EvalIn<Ctx> for Add<A, B>
where
    Add: ApplyIn<Ctx, (A, B)>,
{
    type Output = <Add as ApplyIn<Ctx, (A, B)>>::Output;
}
```

**利用例:**

```rust
type R = EvaluateIn<StrictCtx, Add<U3, U5>>; // U8

// 高階: 文脈を共有して評価戦略を統一
type MapInStrict = MapIn<StrictCtx, Add, [U1, U2]>;
```

**簡潔評注:** 評価戦略を型として管理。同じ式でも文脈で動作を変更可能。

---

#### パターン#21: Kind タグ付き

**特徴:** ApplyKind<K, Arg> で演算空間を分離。Numerical 域と Boolean 域で衝突を避ける。

**Trait定義:**

```rust
pub trait Kind {}
pub struct NumKind;
pub struct BoolKind;
impl Kind for NumKind {}
impl Kind for BoolKind {}

pub trait ApplyKind<K, Arg> { type Output; }

pub struct Add;

impl<A: Eval, B: Eval> ApplyKind<NumKind, (A, B)> for Add
{
    type Output = /* A + B (数値演算) */;
}

impl<A: Eval, B: Eval> Eval for EApp<Add, (A, B)>
where
    Add: ApplyKind<NumKind, (A, B)>,
{
    type Output = <Add as ApplyKind<NumKind, (A, B)>>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<EApp<Add, (U3, U5)>>; // U8 (NumKind)

// 別の演算空間:
impl ApplyKind<BoolKind, (A, B)> for Add {
    type Output = /* A OR B (論理演算) */;
}
```

**簡潔評注:** 演算空間の衝突を避けながら、同じ Op 名で異なる意味を持たせられます。

---

### グループ B: AST + 高階化互換群

---

#### パターン#6: Eval<EApp<Op,Arg>> for ()

**特徴:** EApp の評価を () の trait に実装。AST構造を保持しながら、グローバル評価。

**Trait定義:**

```rust
pub trait Eval { type Output; }

pub struct EApp<Op, Arg>(PhantomData<(Op, Arg)>);

impl<Op: Eval, Arg: Eval> Eval for EApp<Op, Arg>
where
    <Op as Eval>::Output: Apply<<Arg as Eval>::Output>,
{
    type Output = <<Op as Eval>::Output as Apply<<Arg as Eval>::Output>>::Output;
}

// Op の定義は通常どおり
pub struct Add<A, B>(PhantomData<(A, B)>);
impl<A: Eval, B: Eval> Eval for Add<A, B> {
    type Output = /* A + B */;
}
```

**利用例:**

```rust
type R = Evaluate<EApp<Add, (U3, U5)>>; // U8

// 高階: EApp を EApp で包含
type MapExpr = EApp<Map, (Add, [U1, U2])>;
type Result = Evaluate<MapExpr>;

// AST 記法
use typelude::prelude::*;
let proof: () = ... ;  // Evaluate<MapExpr> へのコンパイル時検証
```

**簡潔評注:** AST 構造（EApp）を値として操作でき、高階適用を明示的に表現できます。

---

#### パターン#7: Expr自己評価

**特徴:** Eval trait を Expr 型自身に持たせる。最も直感的な AST 中心モデル。

**Trait定義:**

```rust
pub trait Eval { type Output; }

pub struct And<A, B>(PhantomData<(A, B)>);

impl<A: Eval, B: Eval> Eval for And<A, B> {
    type Output = And<<A as Eval>::Output, <B as Eval>::Output>;
}

pub struct If<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

impl<Cond, Then, Else> Eval for If<Cond, Then, Else>
where
    Cond: Eval,
    <Cond as Eval>::Output: IfHelper<Then, Else>,
{
    type Output = <<Cond as Eval>::Output as IfHelper<Then, Else>>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<And<ELit<True>, ELit<False>>>; // False
type R2 = Evaluate<If<ELit<True>, ELit<U3>, ELit<U5>>>; // U3

// 高階: Expr が自己組織的に評価される
type MapExpr = EMap<Add<ELit<U1>>, [U1, U2, U3]>;
type Result = Evaluate<MapExpr>;
```

**簡潔評注:** AST 型が Eval を直接実装するため、UX が最も自然。デバッグ時に AST 構造が型エラーに表れます。

---

#### パターン#15: AST Lower化

**特徴:** Ast → Core への変換層を明示。記述層と計算層を分離。

**Trait定義:**

```rust
pub trait Lower { type Core; }
pub trait Eval { type Output; }

// AST層
pub struct AstAdd<A, B>(PhantomData<(A, B)>);
impl<A: Lower, B: Lower> Lower for AstAdd<A, B> {
    type Core = CoreAdd<<A as Lower>::Core, <B as Lower>::Core>;
}

// Core層
pub struct CoreAdd<A, B>(PhantomData<(A, B)>);
impl<A: Eval, B: Eval> Eval for CoreAdd<A, B> {
    type Output = /* A + B */;
}

// Eval 入口
impl<T: Lower> Eval for T
where
    <T as Lower>::Core: Eval,
{
    type Output = <<T as Lower>::Core as Eval>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<AstAdd<ELit<U3>, ELit<U5>>>; // U8

// AST で Map を記述、Lower で Core に変換、Eval で実行
type AstMapping = AstMap<AstAdd<U1>, [U1, U2]>;
type R2 = Evaluate<AstMapping>;
```

**簡潔評注:** 記述層と計算層が分離されるため、ユーザーは AST に集中でき、メンテナンスも容易。

---

#### パターン#16: 二段評価

**特徴:** EvalAst + EvalCore で分離。フェーズが型構造に見える。

**Trait定義:**

```rust
pub trait EvalAst { type AstOut; }
pub trait EvalCore { type Out; }

pub struct AstAdd<A, B>(PhantomData<(A, B)>);

impl<A: EvalAst, B: EvalAst> EvalAst for AstAdd<A, B> {
    type AstOut = (A::AstOut, B::AstOut);  // AST 形式のまま
}

impl<A, B> EvalCore for (A, B) {
    type Out = /* A + B */;
}

// 二段評価の入口
impl<T: EvalAst> EvalCore for T
where
    T::AstOut: EvalCore,
{
    type Out = <T::AstOut as EvalCore>::Out;
}
```

**利用例:**

```rust
type AstPhase = <AstAdd<U3, U5> as EvalAst>::AstOut;  // (U3, U5)
type Result = <AstAdd<U3, U5> as EvalCore>::Out;     // U8

// 高階: 各フェーズで異なる処理を挿入可能
```

**簡潔評注:** 評価フェーズが型レベルで可視化される。意味論の透明性が高い。

---

#### パターン#17: Small-step正規化

**特徴:** Step + Normalize の再帰合成。小ステップ実行意味論を型レベルで表現。

**Trait定義:**

```rust
pub trait Step { type Next; }
pub trait IsNormal {}

pub struct Add<A, B>(PhantomData<(A, B)>);

impl<A: Eval, B: Eval> IsNormal for Add<A, B> { }

impl<A: Step, B> Step for Add<A, B> {
    type Next = Add<A::Next, B>;
}

// 正規形に達するまでステップを繰り返し
impl<T> Eval for T
where
    T: NormalizeRecursive,
{
    type Output = <T as NormalizeRecursive>::Final;
}

pub trait NormalizeRecursive {
    type Final;
}

impl<T: IsNormal> NormalizeRecursive for T {
    type Final = T;
}

impl<T: Step> NormalizeRecursive for T
where
    T::Next: NormalizeRecursive,
{
    type Final = <T::Next as NormalizeRecursive>::Final;
}
```

**利用例:**

```rust
type R = Evaluate<Add<ELit<U3>, ELit<U5>>;  // 正規化まで繰り返す
```

**簡潔評注:** 意味論は明瞭だが、再帰が深くなるため rustc の負荷が高。特殊な用途向け。

---

#### パターン#22: ラムダAST+環境

**特徴:** λ計算を直接型で実装。Var/Lam/App + Env で正規的なモデル。

**Trait定義:**

```rust
pub trait Eval { type Output; }
pub struct Env<Bindings>(PhantomData<Bindings>);

pub struct Var<N>(PhantomData<N>);
impl<Bindings, N> Eval for Var<N>
where
    Bindings: Index<N>,
{
    type Output = <Bindings as Index<N>>::Output;
}

pub struct App<F, Arg>(PhantomData<(F, Arg)>);
impl<F, Arg> Eval for App<F, Arg>
where
    F: Eval,
    <F as Eval>::Output: Apply<Arg>,
{
    type Output = <<F as Eval>::Output as Apply<Arg>>::Output;
}

pub struct Lam<Body>(PhantomData<Body>);
impl<Body> Eval for Lam<Body> {
    type Output = LamValue<Body>;  // クロージャ値
}
```

**利用例:**

```rust
// λx.λy.x + y を型で表現
type AddLam = Lam<Lam<App<Add, (Var<1>, Var<0>)>>>;
type AddFn = Evaluate<AddLam>; // LamValue<...>

// 適用
type Result = Evaluate<App<App<AddFn, U3>, U5>>; // U8
```

**簡潔評注:** λ計算に忠実で理論的には正確だが、型評価系が重いため実用性は限定的。教育用途向け。

---

### グループ C: その他

---

#### パターン#2: Self=Arg, ApplyOp forArg

**特徴:** Arg 型が主語になり、Apply<Op> を実装。演算子中心の直感と逆向き。

**Trait定義:**

```rust
pub trait Apply<Op> { type Output; }

pub struct U3;

impl<Op> Apply<Op> for U3 {
    type Output = /* Op, U3 に基づく結果 */;
}

// 具体例: U3 が Add を受け取る
impl Apply<Add> for U3 {
    type Output = Add<U3>;  // 部分適用
}

impl Eval for Add<U3> {
    type Output = AddFn<U3>;
}
```

**利用例:**

```rust
type R = <U3 as Apply<Add>>::Output;  // Add<U3>

// 高階: データがオペレーションを「吸収」する形になる
type Result = Evaluate<Map<U1, [Add, Mul, Div]>>;
```

**簡潔評注:** データ型中心モデルで、関数型パラダイムと逆向き。ドメイン理論向けだが通常の演算には不自然。

---

#### パターン#3: Self=(Op,Arg), Apply for (Op,Arg)

**特徴:** ペア型 (Op, Arg) の trait に Apply を実装。実装単位が組になる。

**Trait定義:**

```rust
pub trait Apply { type Output; }

pub struct Pair<Op, Arg>(PhantomData<(Op, Arg)>);

impl<Op: Eval, Arg: Eval> Apply for Pair<Op, Arg>
where
    <Op as Eval>::Output: ApplyFn<<Arg as Eval>::Output>,
{
    type Output = <<Op as Eval>::Output as ApplyFn<<Arg as Eval>::Output>>::Output;
}

impl<Op, Arg> Eval for Pair<Op, Arg>
where
    Pair<Op, Arg>: Apply,
{
    type Output = <Pair<Op, Arg> as Apply>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<Pair<Add, (U3, U5)>>; // U8

// 高階: ペアをネストさせる
type MapPair = Map<Pair<Add, (X, Y)>, List>;
```

**簡潔評注:** ペア粒度での管理が必要になり、カスタマイズと複雑性のトレードオフが出やすい。

---

#### パターン#9: Arg側GAT (AppliedBy::Apply<Op>)

**特徴:** Arg 型に GAT Apply<Op> を定義。責務がデータ側へ寄る。

**Trait定義:**

```rust
pub trait AppliedBy { type Apply<Op>; }

pub struct U3;
impl AppliedBy for U3 {
    type Apply<Op> = /* Op に基づく適用結果 */;
}

impl AppliedBy for U3 {
    type Apply<Add> = Add<U3>;
    // type Apply<Mul> = Mul<U3>;
    // ...
}

impl<Op> Eval for <U3 as AppliedBy>::Apply<Op> {
    type Output = /* 実行結果 */;
}
```

**利用例:**

```rust
type R = <U3 as AppliedBy>::Apply<Add>; // Add<U3>

// 高階: データ型が「どのオペレーションに対応するか」を定義
```

**簡潔評注:** データ型に演算の責務を寄せるため、関数型プログラミングと相反。汎用ライブラリには不向き。

---

#### パターン#18: 関係述語 (ReducesTo<Op, Arg>)

**特徴:** ReducesTo<Op, Arg> で関係を定義。関数というより与件として扱う。

**Trait定義:**

```rust
pub trait ReducesTo<Op> { type Result; }

pub struct Add<A, B>(PhantomData<(A, B)>);

impl<A: Eval, B: Eval> ReducesTo<U3> for Add<A, B>
where
    <A as Eval>::Output: Addable,
    <B as Eval>::Output: Addable,
{
    type Result = /* A + B */;
}

impl<Expr, Op> Eval for (Expr, Op)
where
    Expr: ReducesTo<Op>,
{
    type Output = <Expr as ReducesTo<Op>>::Result;
}
```

**利用例:**

```rust
type R = <Add<U3, U5> as ReducesTo<U1>>::Result;  // 関係の確立
```

**簡潔評注:** 関係定義として扱いやすいが、関数の「引数」と「適用」の感覚は薄い。形式体系の研究向け。

---

#### パターン#19: CPS継続 (ApplyK<Arg, Cont>)

**特徴:** ApplyK<Arg, Cont> で継続を型パラメータに。制御フロー表現は強いが認知負荷高。

**Trait定義:**

```rust
pub trait ApplyK<Arg, Cont> { type Output; }

pub struct Add;

impl<A: Eval, B: Eval, K> ApplyK<(A, B), K> for Add
where
    K: ContinuationFn,
{
    type Output = <K as ContinuationFn>::Call</* A + B */>;
}

pub trait ContinuationFn {
    type Call<X>;
}

impl Eval for EAppK<Add, (U3, U5), IdContinuation>
where
    Add: ApplyK<(U3, U5), IdContinuation>,
{
    type Output = <Add as ApplyK<(U3, U5), IdContinuation>>::Output;
}
```

**利用例:**

```rust
type R = Evaluate<EAppK<Add, (U3, U5), IdContinuation>>; // U8

// CPS 適用で制御フローを明示的に管理
```

**簡潔評注:** 制御表現に強いが、通常の演算には複雑性が増す。特殊なDSL向け。

---

## グループ間のまとめ

### グループ A 選択時

- 最もシンプルな実装フロー
- 新規 Op の追加が最小限の変更で済む
- trait 境界の理解が容易
- ライブラリとしての拡張性が高い

**推奨:** v2 のメインパターンとして、パターン#1 または #12 を軸に採用

### グループ B 選択時

- AST 記述とデバッグが自然
- 意味論の透明性が高く、フォーマルベリファイに向く
- 学習曲線は緩い
- trait 層が増える傾向

**推奨:** AST → Core 分離が必要な場合、パターン#7 または #15 を軸に採用

### グループ C 検討時

- 特定のドメインでの利点がある
- 一般汎用ライブラリには不適合
- 学習コストが高い

**推奨:** 基本設計では採用せず、機能拡張時に必要に応じて組み込み

---

## 次の検討ステップ

1. **有力候補の選定**
   グループ A: パターン#1 或いは #12 を軸に
   グループ B: パターン#7 或いは #15 を軸に
   組み合わせの検討

2. **境界固定**
   - 多引数の正準形（カリー化かタプル化か）
   - 評価戦略（厳格評価か遅延評価か）
   - If/While の位置づけ（制御構文か通常Op化か）

3. **プロトタイピング**
   選定パターンで Bool, Array, Map(高階) の最小実装を検証

4. **設計文化の確立**
   - 新規Op実装のガイドライン
   - Helper層の公開/非公開境界ルール
   - Trait冗長性チェック機構
