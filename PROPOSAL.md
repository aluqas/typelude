# 型レベル計算モデルの抽象化と分離に関する提案 (Proposal for Abstraction and Separation of Type-Level Computation Models)

## 1. はじめに (Introduction)

本ドキュメントは、`typelude` における型レベル計算モデルの抽象化と、具体的な実装（`std` や `lambda`）の分離、および実装パターンの標準化に関する提案をまとめたものである。

現状、`typelude-core` には「評価エンジン（Eval）」「標準ライブラリ（std）」「ラムダ計算（lambda）」が混在しており、それぞれの境界や役割分担が曖昧な部分がある。本提案では、これらを明確なレイヤー構造として再定義し、今後の拡張性とメンテナビリティを向上させることを目的とする。

## 2. 現状の課題 (Current Issues)

1.  **計算モデルの境界の曖昧さ**: `typenum` 等をラップする「即時評価的なモデル（Direct Model）」と、`LApp` を用いる「遅延評価的なラムダ計算モデル（Lambda Model）」が混在しており、ユーザーや開発者がどちらを選択・実装すべきかの指針が明確でない。
2.  **抽象化の規則の欠如**: `Apply` トレイトの実装において、専用のASTノード（例: `EAdd`）を返す場合と、汎用の `App` を使う場合の使い分けが形式化されていない。
3.  **トレイト境界の複雑化**: 型レベル計算特有の「トレイト境界の爆発（Trait Bound Explosion）」を防ぐための `Eval` パターンの意図が、コード上で明示的なアーキテクチャとして表現されていない。

## 3. アーキテクチャの階層化 (Architectural Layers)

計算モデルを以下の3つのレイヤーに整理することを提案する。

### Layer 0: Kernel (Evaluation Engine)
計算の「メカニズム」のみを提供する最下層。特定のデータ型や演算には依存しない。

*   **`Eval`**: 値への簡約（Reduction）を行うコアトレイト。
*   **`Apply<Args>`**: 構文構築（Syntax Building）および関数適用を行うトレイト。
*   **`App<Op, Args>`**: 汎用的な関数適用を表すASTノード。

### Layer 1: Interfaces (Type Classes)
具体的な実装から切り離された「抽象的な操作」の定義。Haskellの型クラスに近い役割を果たす。
これにより、`std` は「`typenum` を使った実装」の1つに過ぎないという位置付けになる。

*   **`TypeNat`**: 自然数としての振る舞いを定義。
*   **`TypeBool`**: 真偽値代数としての振る舞いを定義 (`And`, `Or`, `Not`)。
*   **`TypeList`**: リスト操作の定義。
*   **`TyFn`**: 型レベル関数の抽象。

### Layer 2: Concrete Models (Implementations)
抽象インターフェースに対する具体的な実装。用途に応じて選択される。

#### Model A: Direct Evaluation Model (`std`)
Rustのエコシステム（`typenum` や `bool` 定数）への直接的な写像を行うモデル。
*   **特徴**: パフォーマンス（コンパイル速度）重視。Rustコンパイラの組み込み演算を可能な限り利用する。
*   **用途**: 数値計算、フラグ管理、配列のインデックス計算など。
*   **実装**: `OpAdd` は `typenum::Add` を呼び出す `EAdd` に展開される。

#### Model B: Lambda Calculus Model (`lambda`)
純粋なラムダ計算に基づくモデル。
*   **特徴**: 表現力重視。カリー化、部分適用、高階関数を自然に扱える。
*   **用途**: 複雑な制御フロー、関数型プログラミングのパターンの適用。
*   **実装**: `LApp` を用いた項の書き換え（Rewriting）。遅延評価（Call-by-Name）や戦略的な簡約が可能。

---

## 4. 実装パターンの定式化 (Standardization of Implementation Patterns)

開発者が新しい演算や型を追加する際に従うべき3つのパターンを定義する。

### Pattern 1: Op-Expr Pattern (Dedicated AST)
頻繁に使用される基本的・プリミティブな演算向け。

*   **構成**:
    *   **Operator**: `struct OpAdd;` (Marker)
    *   **Expression**: `struct EAdd<L, R>;` (AST Node)
*   **動作**:
    *   `OpAdd` は `Apply` を実装し、`EAdd` を返す。
    *   `EAdd` は `Eval` を実装し、計算結果を返す。
*   **利点**: 型シグネチャが短くなる（`App<OpAdd, ...>` vs `EAdd<...>`）。デバッグ時に型名から意図を読み取りやすい。特殊化や最適化が容易。
*   **適用基準**: `std` の基本演算（算術、論理、比較）。

### Pattern 2: Generic App Pattern (Unified AST)
ユーザー定義関数や、組み合わせによって作られる高階関数向け。

*   **構成**:
    *   **Operator**: `struct MyFunc;`
*   **動作**:
    *   `MyFunc` は `Apply` を実装するが、特定の `EMyFunc` 構造体は定義しない。
    *   適用結果は `App<MyFunc, Args>` として表現される（あるいは `Apply` の中で別の既存ASTを返す）。
*   **利点**: ボイラープレート（AST構造体の定義）が不要。柔軟性が高い。
*   **適用基準**: ライブラリの拡張部分、複雑なロジックの合成。

### Pattern 3: Bridge Pattern
異なるモデル間や、評価戦略の制御を行うためのパターン。

*   **`ELit<T>`**: 値 `T` を式（Expression）の世界に持ち上げる。Evaluation は恒等写像。
*   **`ECall<F, A>`**: Call-by-Value の強制。引数 `A` を評価してから関数 `F` に適用したい場合に使用。
*   **`ELazyCall<F, A>`**: Call-by-Name の明示。引数 `A` を未評価のまま渡す。

---

## 5. 技術的背景: RustコンパイラとEvalパターン

なぜこのような回りくどい設計（ApplyしてEvalする）が必要なのか、その技術的背景を明記する。

### トレイト境界の隠蔽 (Hiding Trait Bounds)
Rustのトレイトソルバは、型エイリアスや関連型を展開する際、定義側で指定された `where` 句（制約）が満たされているかを厳密にチェックする。
演算が連鎖すると、上位の型定義には下位の全ての演算に必要な制約（例: `A: Add<B>, A+B: Mul<C>, ...`）を列挙する必要が生じ、**「トレイト境界の爆発（Trait Bound Explosion）」**が発生する。

`Eval` パターンはこれを解決する：
1.  **遅延展開**: `Apply` は単に AST（構造体）を構築するだけで、計算（`Eval`）は行わない。この時点では深い制約チェックは発生しない。
2.  **制約の局所化**: 実際に計算が必要な場所（`Evaluate<T>` を呼ぶ場所）でのみ `T: Eval` を要求する。`Eval` の実装内部にある複雑な `where` 句は、外部のAPI利用からは隠蔽される。

### 再帰制限への対処 (Handling Recursion Limits)
型レベル計算はコンパイラの再帰的な型解決に依存するため、`#![recursion_limit]` に達しやすい。
*   **Direct Model**: `typenum` 等の既存実装に委譲するため、再帰深度は比較的浅く済む。
*   **Lambda Model**: 項書き換えが深くネストするため、再帰制限にヒットしやすい。
    *   対策: ステップ実行（`RunStep`）や、`Eval` を一度切って `App` で再構築する（トランポリンのような挙動）工夫が必要になる場合がある。

## 6. 今後のロードマップ案

1.  **Refactoring**: `std` 内の演算定義を `define_arith_op!` 等のマクロを用いて "Op-Expr Pattern" に統一し、`TypeNat` 等のインターフェースへの依存に切り替える。
2.  **Documentation**: 各レイヤーとパターンの役割を明記した `CONTRIBUTING.md` または `ARCHITECTURE.md` の整備。
3.  **Expansion**: `lambda` モデルにおける `Monad` などの高レベル抽象の実装拡充。

---
*Proposed by Jules (AI Agent) - Session ID: [Current Session]*
