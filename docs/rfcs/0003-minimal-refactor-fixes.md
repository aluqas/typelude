---
id: rfc-0003
title: "Minimal Refactor Fixes"
status: draft
created: 2026-01-05
author:
---

# RFC 0003: Minimal Refactor Fixes

- Sealedの廃止
  - クレートをまたぐと効果が薄いのもあるし、シンプルに邪魔になってきた
- コメント
  - `// --------`のような視覚的な区切りの廃止
  - 一般的によく言われる無駄なコメントの最小化と、doccommentの充実
    - ただし、本プロジェクトの特性を加味し、コメントは積極的につける
- ドキュメント
  - LLMが扱いやすいアイデア・作業状況・アーキテクチャ情報管理
- ガードレール機能の最大化
  - テストフレームワークの設計
    - 統合テストへの移譲と、ユニットテストのミニマル効果最大化
    - `cargo-mutsnts`
    - `cargo-llvm-cov`
    - `cargo-nextest`
    - proptest
  - clippy/fmt/プロファイリングの設定
- 実装周り
  - パス/API/公開制御
    - lib/mod.rsによるfacadeパターンを意識した設計の一貫性
    - API/アーキテクチャ/モジュール構造の安定化に向けた調整
  - ASTへの一本化
    - Apply+OpのAST化は都度エイリアスが必要だが、逆は一つのエイリアスでよい
    - `type Apply<T, U> = <T<U> as Eval>::Output`
  - マクロを利用した実装への統一
    - 実装・デザインパターンの一貫化の保証

# Refactoring Design Decisions (Addendum)

## 1. Directory Structure & Module Organization

**方針**: `std` クレート内のモジュール構造を、役割ベースで明確に分離・再配置する。

- **`col` / `prim` モジュールへの再編**:
  - **`col` (Collections)**: `tree`, `map` などのデータ構造。
  - **`prim` (Primitives)**: `int`, `bool`, `option` などの原始的な型。
  - ユーザーの好みに合わせ、`collections` / `primitives` ではなく短縮形を採用する。

## 2. Naming Conventions

**方針**: Rustのエコシステム標準に準拠し、冗長なPrefixを廃止する。

- **`Ty` Prefixの廃止**:
  - `Ty` (e.g., `TyNode`, `TyLeaf`) はコンテキストとして自明であるため削除する。
  - `col::tree::TyNode` -> `col::tree::Node`
  - `col::map::TyMap` -> `col::map::Map`
  - 名前空間 (`std::col::tree` 等) によって衝突を回避し、識別する。

- **`Op`, `E`/`L` Prefixの維持**:
  - **`Op*`**: 「操作 (Operation)」と「型 (Type)」を区別するため維持する (`OpInsert` vs `Insert` trait)。
  - **`E*` / `L*`**: AST (`Expression`) や Lambda 表現であることを明示するため、内部表現・デバッグ観点から維持する。

## 3. Standardization & API Alignment

**方針**: ユーザー体験を `std` コレクションに近づける。

- **Trait名の変更**:
  - `GenericArray` や `std::collections` に倣い、一般的な名称を採用する。
  - `TreeInsert` -> `col::tree::Insert`
  - `TreeContains` -> `col::tree::ContainsKey`

## 4. Operator Strategy

**方針**: 独自 `Op` トレイトを正とし、`std::ops` は補助的な位置づけとする。

- **独自定義 (`Op`) の優位性**:
  - `std::ops` は算術演算等に限られ、`Insert` や `Fold` 等のコレクション操作をカバーできない。
  - `def_op!` マクロを用いた独自 `Op` 定義を標準とする。
  - `std::ops` (e.g. `+`) は、あくまで `OpAdd` を呼び出す糖衣構文 (Syntax Sugar) として実装可能な場合のみ提供する。

## 5. Sealed Pattern Abolition

**方針**: `Sealed` トレイトパターンを廃止する。

- **理由**:
  - クレートを跨ぐと効果が限定的であり、ユーザー定義の拡張を不必要に阻害する場合がある。
  - コードベースにおけるノイズとなり、メンテナンス性を下げる。
- **対応**:
  - 既存の `trait Sealed` 定義および、トレイト境界 (`T: Sealed`) を削除する。

## 6. Testing & Documentation Standards

**方針**: 型レベルプログラミング特有の「コンパイル時間」「エラーメッセージ」「ロジック正当性」を多角的に保証する。

- **Documentation**:
  - `///` ドキュメントコメントを必須化。区切り線 (`// ----`) は廃止する。
  - **Compile-Fail Examples**: ドキュメント中に `compile_fail` ブロックを積極的に用い、「コンパイルされるべきでないコード」を例示する。

- **Testing Framework**:
  - **Authentication (正当性検証)**:
    - **Property-Based Testing (`proptest`)**:
      - 型レベル演算の結果が、値レベルの演算結果と一致することをランダム入力で検証する。
      - 例: `eval(TypeAdd<A, B>) == val(A) + val(B)`
    - **UI Testing (`trybuild`)**:
      - **Compile-Fail Tests**: 不正な型操作に対して、期待通りの「わかりやすいエラーメッセージ」が出ているかを機械的にテストする。
      - **Compile-Pass Tests**: 複雑な型パズルが意図通りコンパイルを通ることを保証する。

  - **Profiling (パフォーマンス・自明性検証)**:
    - **Compilation Time**:
      - `cargo build --timings` をCIに導入し、コンパイル時間の回帰を監視する。
    - **Instantiations & Bloat**:
      - `cargo llvm-lines` を用いて、過剰な単相化 (Monomorphization) が発生していないか定期チェックする手順を確立する。
    - **Deep Profiling**:
      - 開発者向けに `rustc -Z self-profile` を用いたプロファイリングガイドを整備し、ボトルネック調査の手法を明確化する。

## 7. Typelude Core Refactoring

**方針**: `typelude-core` を純粋な抽象層とし、具体的な実装や依存 (`typenum` 等) を `typelude-std` へ移譲する。

- **`impls.rs` の移動**:
  - `typelude-core/src/impls.rs` (`typenum` への `Eval` 実装) を `typelude-std/src/std/impls.rs` へ移動する。
  - これにより `typelude-core` の `typenum` 依存を削除し、軽量なTrait定義のみのクレートとする。

- **`bridge` モジュールの見直し (`expr`)**:
  - モジュール名を `bridge` から `expr` (Expressions) へ変更する。
  - ASTノードの命名をより正確な用語へ変更する:
    - `ECall` -> `EApp` (Evaluation Application)
    - `ELazyCall` -> `ELazyApp` (Lazy Application)
    - `EPureApp` -> `EPure` (Pure Expression wrapper)
    - `ELit` は維持 (Literal/Input lifting)。

- **Application Semantics**:
  - **`trait Apply<A>`**: 関数適用能力を定義する基本トレイト。
  - **`type App<F, A>`**: `<F as Apply<A>>::Output` のType Alias。
  - **`EApp<Ef, Ea>` (Call-by-Value)**:
    - 評価手順: `Ef`評価(F) -> `Ea`評価(A) -> `App<F, A>`
    - 通常の関数適用AST。引数も評価してから渡す。
  - **`ELazyApp<Ef, Ea>` (Call-by-Name)**:
    - 評価手順: `Ef`評価(F) -> `App<F, Ea>`
    - 制御構文や遅延評価用。引数 `Ea` を未評価のままASTとして渡す。
