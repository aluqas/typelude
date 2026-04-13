# 共通インターフェース設計方針

## ステータス

採用方針メモ。`v2_memo.md` と `v2_HIGHER_ORDER_API_PATTERNS.md` の検討を踏まえた、現時点の設計判断をまとめる。

## 要約

Typelude では、以下を分離して設計する。

- **共通計算インターフェース**: `Eval`
- **高階適用インターフェース**: `Op<Args>` + `Apply<Op, Args>` 相当の AST
- **公開意味論インターフェース**: `std::ops` 風の capability trait
  - 例: `Add<Rhs> for Lhs`, `Sub<Rhs> for Lhs`, `Not for T`, `Get<Idx> for Col`

結論として、**`Eval` / `Apply` の共通化は行うが、`Add` / `OpAdd` の意味論そのものを ownership 境界を越えて共通化しようとはしない**。

## 背景

高階 API（`While`, `Map`, `Fold` など）では、演算子や述語を first-class に扱う共通 ABI が必要になる。一方で、各演算の意味論を downstream が自由に拡張できないと、ライブラリとしての汎用性が落ちる。

ここで問題になるのが orphan rule / coherence である。

- `Eval` や `Apply` のような共通 ABI は統一できる
- しかし `Add` のような意味論 trait を「全クレートで同一のもの」として自由に合流させるのは難しい
- Rust は「意味論の所有者」を明確にする方向で coherence を設計している

したがって、**共通化するのは計算 ABI であり、意味論そのものではない**

## 基本方針

### 1. `typelude-std` が arity を規定する

`typelude-std` は標準演算の意味論を持つ層として、演算ごとの arity を規定する。

- 二項演算: `Add<Rhs>`, `Sub<Rhs>`, `Mul<Rhs>`, `Div<Rhs>`, `Rem<Rhs>`, `Eq<Rhs>`, ...
- 単項演算: `Not`, `Neg`, ...
- コレクション演算: `Len`, `Head`, `Tail`, `Get<Idx>`, `Set<Idx, Val>`, ...

これは `std::ops` 風の surface を typelude が**自前で所有する**、という意味である。`typenum` などが内部で `std::ops::Add` を使っていても、typelude の公開インターフェースまでそれに従う必要はない。

### 2. capability trait が公開意味論の本体

演算の公開意味論は self-based capability trait で表す。

```rust
pub trait Add<Rhs> {
    type Output;
}
```

これにより downstream は自分の local type を主語にして意味論を追加できる。

```rust
impl<Rhs> Add<Rhs> for MyType {
    type Output = ...;
}
```

### 3. AST は `Eval` を通じて capability trait に委譲する

演算 AST は利用者向けの記述面である。意味論そのものは capability trait に集約する。

```rust
pub struct EAdd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EAdd<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: Add<Evaluate<Rhs>>,
{
    type Output = <Evaluate<Lhs> as Add<Evaluate<Rhs>>>::Output;
}
```

### 4. `OpX` は高階 API のための first-class symbol

`OpAdd`, `OpGet`, `OpLen` などは、高階 API で演算を受け渡すための first-class symbol である。  
意味論は `OpX` 自体に直接分散させず、capability trait に委譲する。

```rust
pub struct OpAdd;

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpAdd
where
    Lhs: Add<Rhs>,
{
    type Output = <Lhs as Add<Rhs>>::Output;
}
```

### 5. 高階 API は `OpX` を見る

`While`, `Map`, `Fold` などの高階 API は、個別演算 trait を直接見るのではなく、`Op<Args>` / `Apply<Op, Args>` を通じて演算子を受け取る。

これにより、個別演算の意味論と高階適用 ABI を分離できる。

## 採用パターン

### 公開レイヤ

1. **意味論レイヤ**
   - `Add<Rhs> for Lhs`
   - `Sub<Rhs> for Lhs`
   - `Len for Col`
   - `Get<Idx> for Col`

2. **AST レイヤ**
   - `EAdd<Lhs, Rhs>`
   - `EGet<Col, Idx>`
   - `If<Cond, Then, Else>`
   - `While<Pred, Step, State>`

3. **高階演算子レイヤ**
   - `OpAdd`
   - `OpGet`
   - `OpLen`
   - `OpIf`
   - `OpWhile`

### 依存方向

- AST は capability trait か `OpX` に委譲する
- `OpX` は capability trait に委譲する
- capability trait は公開意味論の終端である

## orphan rule / coherence についての整理

今回の議論の核心は次の点にある。

- `Eval` / `Apply` のような共通 ABI は typelude が所有すれば統一できる
- しかし、異なるクレートの `Add` を「同じ意味論 trait」として後から自由に合流させることは難しい
- これは API の偶然ではなく、Rust の coherence model の本質に触れている

したがって、Typelude は以下を採る。

- **共通化するもの**: 計算 ABI (`Eval`, `Op`, `Apply`)
- **局所化するもの**: 意味論 (`Add`, `Sub`, `Get`, ...)

## downstream 向けの実装規約

### 既存の canonical op を自分の型に対応させたい場合

capability trait を実装する。

```rust
impl<Rhs> Add<Rhs> for MyLocalType {
    type Output = ...;
}
```

### 新しい first-class operation を作りたい場合

local な `Op` symbol を定義して `Op<Args>` を実装する。

```rust
pub struct OpMyThing;

impl<Args> Op<Args> for OpMyThing {
    type Output = ...;
}
```

### AST を追加したい場合

`Eval` を実装し、必要なら既存の capability trait または `OpX` へ lower する。

## rejected / 非採用方針

### `Add` 自体を universal callable interface にする

非採用。

- `Add` は AST、primitive semantics、first-class callable の 3 役を背負いがち
- 役割が混ざると API が不安定になる
- 共通化すべきなのは `Add` ではなく `Eval` / `Apply`

### `Add for LocalWrapper<Args>` を共通インターフェースにする

原理的には可能だが、設計上は採用しない。

- 本質的に `Eval` の限定版・別名になりやすい
- capability trait としての `Add` の意味が薄くなる
- 演算ごとの公開意味論より、式 calculus の別表現に寄りすぎる

### `Apply<Op> for Args` を universal extension point にする

限定的には使えるが、公開意味論フックの主軸にはしない。

- 高階 ABI としては有効
- ただし意味論フックまでそこへ寄せると、capability trait の責務と競合する
- downstream 拡張点としては self-based capability trait の方が読みやすく、自然である

## 実装テンプレート

### capability trait

```rust
pub trait Add<Rhs> {
    type Output;
}
```

### AST

```rust
pub struct EAdd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EAdd<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: Add<Evaluate<Rhs>>,
{
    type Output = <Evaluate<Lhs> as Add<Evaluate<Rhs>>>::Output;
}
```

### first-class op

```rust
pub struct OpAdd;

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpAdd
where
    Lhs: Add<Rhs>,
{
    type Output = <Lhs as Add<Rhs>>::Output;
}
```

## 設計原則の一文要約

**Typelude は、意味論を capability trait に集約し、AST と first-class op をその二つの view として提供する。共通化するのは計算 ABI であり、意味論そのものではない。**
