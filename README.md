# Typelude - Rust Type Utilities

// TODO: multi-language support

Typeludeは、Rustの型システムのための高度な論理演算/ユーティリティを提供します。

## アーキテクチャ

- 値: `TyTrue`や`TyFalse`など。
- 型: `AsBool`などのTrait。

### 演算・関数部分について

#### トレイト

これらはトレイトとして存在。

- **Rustの型システムでうまく条件分岐できるのは、implターゲット/Output:出力する型をトレイト境界によって場合によって分岐できるトレイトしかない。**

1. `impl<Lhs, Rhs> TyAnd<Rhs> for Lhs`により、`where Lhs: TyAnd<Rhs, Output = TyTrue>`のようなLhsがSelfパターン。
2. `where (): TyAnd<Lhs, Rhs, Output = TyTrue>`のように、`()`またはTarget / StateオブジェクトをSelfにするパターン。

- `TyAndTrue<Lhs, Rhs>`のようなOutputのショートカットトレイトは提供しない。
  - Wrapを提供するとあらゆるパターンが考えられるので散らかる。なんかどこかでマクロでも提供しようか。
- 2は基本的に実装しないが、ショートカットと合わせて後ほどpredicateトレイトパターンで使う可能性がある。

3. 特殊: `Bool`/演算結果(Output)そのものををSelfとして、`where IsTrue: TyAnd<Lhs, Rhs>`のようなパターン

- 分かりづらいので使わないが...コンパイラに有効な時がありそうなので一応メモ。

#### 型エイリアス・構造体

1. `type And<Lhs, Rhs> = <Lhs as TyAnd<Rhs>>::Output`のような「純粋に値を返す関数」パターン。

- 基本的には上で実装したもののショートカットにすぎない
- 直感的に使え、かつ内包できるので純粋に関数っぽく使える。
- **これだけだとトレイト境界伝搬の地獄がヤバい。**
  - 例えば、`Or<A, And<B, C>>`をしようとすると、`where A: TyOr<And<B, C>>, B: TyAnd<C>`のようなことをしなければならず
  - ただし、この境界を断ち切ることがEvalパターンなどでできるので、実装側に集約して回避する。
- 公開は基本的にこれを使う。

2. `struct Eval`パターン

- And<L,R> / Or<L,R> / Not<T> を「構文木 struct」にする
- Eval トレイトで `Eval for And<L,R> { type Output = ... }` のように計算ロジックと 境界bound を全部ここに閉じ込める
- 呼び出し側の境界は基本 `Logic<A,B,C>: Eval` だけを見る

3. predicateトレイトパターン

-

##

- Expression→追加のドメイン固有トレイトが必要
- Function→既存のEvaluableトレイトのみで実装可能
