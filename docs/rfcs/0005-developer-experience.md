---
id: rfc-0005
title: "Type-Level Developer Experience Enhancements"
status: draft
created: 2026-01-06
author: saqula
---

# RFC-0005: Type-Level Developer Experience Enhancements

## Summary

型レベルプログラミング（特にType-Level VM）の開発・デバッグ体験を向上させるための機能群を提案する。現在のトレース（履歴）機能を拡張し、インラインアサーション、コンパイル時ブレークポイント、ガス制限、ログ機能、および実行時ブリッジを導入する。

## Motivation

型レベルVMは強力だが、「書くときは楽しいが、直すときは地獄」になりがちである。

- **エラー特定が困難**: 長いプログラムで型エラーが発生した際、どのステップで問題が起きたか特定するのが極めて難しい
- **中間状態の確認が困難**: `assert_type_eq!`はテストの最後でしか使えず、途中経過を確認できない
- **無限ループ対策の欠如**: 意図せぬ無限ループはRustの`recursion_limit`に依存しており、VM側で制御できない
- **型エラーメッセージの可読性**: 長い履歴を含むエラーメッセージは人間が読むのに適していない

これらを解決し、型レベルプログラミングの実用性を飛躍的に向上させる。

## Detailed Design

### 1. `OpAssert<Predicate>` - インライン・スタティック・アサーション

プログラム中に型レベルのアサーションを埋め込み、実行時ではなくコンパイル時に検証する。

```rust
/// Type-level assertion operator
pub struct OpAssert<Predicate>(PhantomData<Predicate>);

/// Example predicates
pub struct StackIs<Expected>;
pub struct MemoryIs<Expected>;
pub struct LocalsLengthIs<N>;
```

**使用例:**

```rust
type Prog = tyarray![
    OpPush<ELit<U3>>,
    OpPush<ELit<U2>>,
    OpAssert<StackIs<tyarray![ELit<U2>, ELit<U3>]>>,  // このチェックポイント
    OpAdd,
    OpAssert<StackIs<tyarray![ELit<U5>]>>,             // 加算後のチェック
];
```

**実装方針:**

- `Execute/TracedExecute`トレイトで、`Predicate`が満たされない場合は型エラーになるように実装
- 満たされる場合は状態をそのまま通過させる（履歴には追加可能）

---

### 2. `OpDump` - コンパイル時ブレークポイント

Rust 1.78+の`#[diagnostic::on_unimplemented]`属性を活用し、意図的にコンパイルエラーを発生させて、その時点の状態をエラーメッセージに出力する。

```rust
/// Intentional compile-time breakpoint
pub struct OpDump;

// Execute実装を意図的に欠落させ、diagnosticでカスタムエラーを表示
#[diagnostic::on_unimplemented(
    message = "=== DUMP ===",
    label = "Stack: {Stack}, Locals: {Locals}, Memory: {Memory}",
    note = "This is a debug breakpoint. Remove OpDump to continue."
)]
pub trait ExecuteDump<Stack, Locals, Memory, CallStack, RestProg> {}
```

**使用シナリオ:**

- 複雑なプログラムのどこかで型エラーが出る場合、段階的に`OpDump`を挿入して「ここまでは正しいか？」を確認

---

### 3. `Gas` Metering - 再帰深度の自己制御

MachineStateにGas（残り実行ステップ数）を持たせ、VM側で再帰深度を制御する。

```rust
pub struct MachineStateWithGas<Stack, Locals, Memory, CallStack, Program, Gas>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program, Gas)>
);

// Gas = U0 になった時点で強制終了
```

**動作:**

- 各ステップ実行時に`Gas`を1減算
- `Gas = U0`でプログラムが残っていても終了（正常終了 or 特殊終了状態）
- ユーザーは`U256`や`U1024`などで上限を設定

**メリット:**

- コンパイラのハングアップを防止
- 計算量の見積もり検証（「このアルゴリズムはNステップ以内で終わるはず」のテスト）

---

### 4. `OpLog<Label>` - コンパイル時ロギング

型レベルでのラベル付きログをHistoryに残す。

```rust
pub struct OpLog<Label>;

// Historyに追加される型
pub struct LogEntry<Label>;
```

**使用例:**

```rust
type Prog = tyarray![
    OpLog<"Init">,
    OpPush<ELit<U1>>,
    OpLog<"AfterPush">,
    OpAdd,
    OpLog<"AfterAdd">,
];
// History: [LogEntry<"AfterAdd">, OpAdd, LogEntry<"AfterPush">, OpPush<...>, LogEntry<"Init">]
```

**実装上の注意:**

- `const generics`で`&'static str`を型パラメータに持つ（`feature = "adt_const_params"`が必要な可能性あり）
- Stableでは、代わりにマーカー型（`struct LogInit;`, `struct LogAfterAdd;`）を使う

---

### 5. Type-to-Const Bridge - 実行時へのブリッジ

型レベルVMの計算結果を、ランタイムの定数値として取り出す機能の強化。

```rust
/// 型レベルのスタック状態をランタイムのVec<i64>として取り出す
pub trait ReifyStack {
    fn reify() -> Vec<i64>;
}

/// 型レベルのメモリ状態をランタイムの配列として取り出す
pub trait ReifyMemory<const N: usize> {
    fn reify() -> [i64; N];
}
```

**応用例:**

- コンパイル時に計算したルックアップテーブルをバイナリに焼き付ける
- 型レベルで検証済みの初期状態をランタイムで利用する

---

### 6. Unified State Architecture（将来検討）

通常の`MachineState`とトレース付きの`TracedMachineState`を、単一のジェネリックな状態型に統一する。

```rust
// 履歴ポリシーをジェネリックにする
pub struct MachineState<Stack, Locals, Memory, CallStack, Program, Listener>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program, Listener)>
);

// Listenerの実装例
pub struct NoTrace;           // 何も記録しない（高速）
pub struct FullTrace;         // 全命令を記録
pub struct BranchOnlyTrace;   // 分岐とメモリ操作のみ記録（軽量）
```

## Theoretical Foundation: Effect System Perspective

本RFCで提案する機能群は、個別のアドホックなデバッグ機能ではなく、**モナド/エフェクトシステム**として統一的に捉えることができる。これはRFC-0002（Monad State Management）で検討されている抽象化と密接に関連する。

### エフェクトとしての分類

| 機能                | 対応するモナド/エフェクト | 意味論                     |
| ------------------- | ------------------------- | -------------------------- |
| **History (Trace)** | Writer モナド             | 実行ログの蓄積（追記のみ） |
| **OpLog**           | Writer モナド             | 構造化されたラベル付きログ |
| **OpAssert**        | Either / Validation       | 失敗パスの早期検出・分岐   |
| **OpDump**          | Abort with Context        | コンテキスト付き中断       |
| **Gas Metering**    | State + 有界再帰          | リソース消費の追跡と制限   |

### 統合設計: `Effect<...>` による合成

これらの副作用を型パラメータとして合成可能にすることで、用途に応じたプロファイルを切り替えられる：

```rust
/// エフェクトスタックを持つ汎用MachineState
pub struct MachineState<Stack, Locals, Memory, CallStack, Program, Effects>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program, Effects)>
);

/// エフェクトの合成
pub struct Effect<E1, E2>;

/// 個別エフェクトの定義
pub struct Pure;                    // 副作用なし
pub struct Writer<Log>;             // ログ蓄積
pub struct Validator<Assertions>;   // アサーション検証
pub struct ResourceBound<Gas>;      // リソース制限
```

### プロファイル例

```rust
/// 純粋計算モード（最速）
type PureExecution = Effect<Pure, Pure>;

/// デバッグモード（履歴 + アサーション）
type DebugExecution = Effect<Writer<FullHistory>, Validator<Strict>>;

/// プロダクションモード（リソース制限のみ）
type ProductionExecution = Effect<ResourceBound<U1024>, Pure>;

/// 軽量トレースモード（分岐のみ記録）
type LightTraceExecution = Effect<Writer<BranchOnlyHistory>, Pure>;
```

### RFC-0002との関係

RFC-0002で定義予定の`TypeMonad`トレイトファミリー（`TypeFunctor`, `TypeApplicative`, `TypeMonad`）は、このエフェクトシステムの基盤となる。

- **bind/flatMap**: エフェクト付き計算の連鎖
- **pure/return**: 純粋な値のエフェクトへの持ち上げ
- **run**: エフェクトの解釈・実行

本RFCの各機能は、RFC-0002のモナド抽象化が完成した後に、より原理的な形で再実装できる可能性がある。

### 将来展望: Algebraic Effects

さらに進んだ設計として、Algebraic Effects（代数的効果）パターンの導入も検討できる：

```rust
/// エフェクトハンドラー
pub trait EffectHandler<E, State> {
    type Result;
    fn handle(state: State) -> Self::Result;
}

/// 例: Logエフェクトのハンドラー
impl<Log, S, L, M, C, P> EffectHandler<Writer<Log>, MachineState<S, L, M, C, P, _>>
    for DefaultLogHandler
{
    type Result = MachineState<S, L, M, C, P, Array<Log, History>>;
    // ...
}
```

これにより、同一のVMプログラムに対して異なるハンドラーを差し込むことで、振る舞いを変更できる。

## Alternatives Considered

### `println!`デバッグの強化（却下）

- 型レベルでは`println!`は使えない
- `Trace::fmt()`による文字列化は可能だが、コンパイル時には確認できない

### 外部ツールによるAST解析（却下）

- proc_macroでプログラムを解析してトレースを出力するアプローチ
- 複雑すぎて保守コストが高い

## Unresolved Questions

1. **`OpDump`の`#[diagnostic::on_unimplemented]`で型パラメータを展開できるか？**
   - Rust 1.78+の仕様確認が必要

2. **`OpLog`で`const &'static str`を型パラメータにするための最小feature構成は？**
   - `adt_const_params`の安定化状況次第

3. **`Gas`メータリングで、Gas切れ時の挙動（正常終了 vs エラー型）はどうすべきか？**
   - ユースケースに応じて選択可能にするか、固定にするか

4. **Unified State Architectureの`Listener`トレイトの設計**
   - 既存の`Execute` / `TracedExecute`との互換性維持方法

## Implementation Notes

### 優先度

1. **High**: `OpAssert` - 最も実用的で実装難易度も低い
2. **High**: `OpDump` - Diagnostic属性の調査後に実装
3. **Medium**: Gas Metering - 状態型の変更が必要
4. **Medium**: Type-to-Const Bridge - 既存の`Reify`トレイトの拡張
5. **Low**: `OpLog` - nightlyフィーチャへの依存度が高い可能性
6. **Future**: Unified State - 大規模リファクタリングが必要

### 関連ファイル

- `crates/typelude-vm/src/machine/trace/execute.rs` - トレース実行ロジック
- `crates/typelude-vm/src/machine/instruction.rs` - 命令定義
- `crates/typelude-std/src/std/reify.rs` - Reifyトレイト
