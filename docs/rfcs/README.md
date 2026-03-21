# RFCs

typeludeの設計提案（Request for Comments）の一覧。

## 一覧

| ID | タイトル | ステータス | 作成日 |
| --- | --- | --- | --- |
| [RFC-0004](./0004-advanced-collections.md) | Advanced Type-Level Collections | draft | 2026-01-06 |
| [RFC-0005](./0005-developer-experience.md) | Type-Level Developer Experience Enhancements | draft | 2026-01-06 |
| [RFC-0006](./0006-primitive-abstraction.md) | Primitive Type Abstraction via Shared Traits | draft | 2026-01-06 |

## ステータス定義

| ステータス | 意味 |
| --- | --- |
| `draft` | 提案中・議論中 |
| `accepted` | 実装方針が確定 |
| `implemented` | 実装済み |
| `superseded` | 別のRFCに置き換えられた |
| `rejected` | 採用しないと決定 |

## RFC概要

### RFC-0004: Advanced Type-Level Collections

`BitSet`（O(1)セット演算）、`Queue`/`Deque`（償却O(1)）、`TupleRecord`（名前付きフィールド）、`HashMap`/`Trie`（ハッシュ・プレフィックス検索）などの高度なコレクション型の追加提案。

### RFC-0005: Type-Level Developer Experience Enhancements

型レベルVMの開発・デバッグ体験の改善提案。`OpAssert`（インラインアサーション）、`OpDump`（コンパイル時ブレークポイント）、Gasメータリング、`OpLog`（コンパイル時ロギング）、Unified State Architectureを含む。

### RFC-0006: Primitive Type Abstraction via Shared Traits

`Iterable`/`Foldable`/`Mappable` などの共通トレイトによるプリミティブ型の抽象化提案。CoreIRの正準化（`typenum`→数値、`tstr`→文字列、`Array/Nil`→リスト）の方針も含む。
