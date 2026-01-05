---
id: rfc-0006
title: "Primitive Type Abstraction via Shared Traits"
status: draft
created: 2026-01-06
author: saqula
---

## Summary

共通抽象トレイトレイヤーを導入し、`Array`/`TreeArray`/`TreeMap`/`Map`/`TyStr`/`typenum`に対して`Foldable`/`Mappable`/`Iterable`等の操作を統一的に適用可能にする。

---

## Implementation Feasibility Analysis

### 既存パターンの分析

#### 1. Array高階操作パターン

```rust
// MapHelper: Op を各要素に適用
pub trait MapHelper<Op> {
    type Output: IsList;
}
impl<Op> MapHelper<Op> for Nil { type Output = Nil; }
impl<Op, H, T> MapHelper<Op> for Array<H, T>
where Op: Apply<H>, ... {
    type Output = Array<Evaluate<Op::Output>, <T as MapHelper<Op>>::Output>;
}

// FoldHelper: Acc を累積
pub trait FoldHelper<Op, Acc> { type Output; }
impl<Op, Acc> FoldHelper<Op, Acc> for Nil { type Output = Acc; }
impl<Op, Acc, H, T> FoldHelper<Op, Acc> for Array<H, T>
where Op: Apply<ECons<Acc, ECons<H, ENil>>>, ... {
    type Output = <T as FoldHelper<Op, Evaluate<Op::Output>>>::Output;
}
```

**観察**: `Nil`/`Array<H, T>`の再帰パターンで、`Op: Apply<...>` 境界が共通。

#### 2. Tree → List変換パターン

```rust
// TreeArrayToList: in-order traversal
pub trait TreeArrayToList {
    type Output: IsList;
}
impl TreeArrayToList for Nil { type Output = array::Nil; }
impl<V, L, R> TreeArrayToList for TreeArray<V, L, R>
where L: TreeArrayToList, R: TreeArrayToList,
      <L as TreeArrayToList>::Output: Concat<Array<V, <R as TreeArrayToList>::Output>>
{
    type Output = <<L as TreeArrayToList>::Output as Concat<...>>::Output;
}
```

**観察**: `ToList`で中間表現に変換し、Array操作を再利用可能。

#### 3. Map (Associative) パターン

```rust
// MapKeys / MapValues: 要素抽出
pub trait MapKeys { type Output: IsList; }
impl MapKeys for Nil { type Output = array::Nil; }
impl<K, V, T> MapKeys for Map<K, V, T> {
    type Output = Array<K, <T as MapKeys>::Output>;
}
```

**観察**: 構造は`IsList`に準拠し、`Concat`等を直接利用可能。

---

## Type System Feasibility

### ✅ 実装可能: Iterable (ToArray)

```rust
pub trait Iterable {
    type Item;
    type ToArray: IsList;
}

// Array: 自明
impl<H, T: IsList> Iterable for Array<H, T> {
    type Item = H;
    type ToArray = Self;
}

// TreeArray: TreeArrayToList経由
impl<V, L: IsTreeArray, R: IsTreeArray> Iterable for TreeArray<V, L, R>
where Self: TreeArrayToList {
    type Item = V;
    type ToArray = <Self as TreeArrayToList>::Output;
}

// TreeMap: TreeMapToList経由
impl<K, V, L: IsTreeMap, R: IsTreeMap> Iterable for TreeMap<K, V, L, R>
where Self: TreeMapToList {
    type Item = (K, V);
    type ToArray = <Self as TreeMapToList>::Output;
}

// Map (Associative): 直接変換可能
impl<K, V, T: TypeMap + Iterable> Iterable for Map<K, V, T> {
    type Item = (K, V);
    type ToArray = Array<(K, V), <T as Iterable>::ToArray>;
}
```

### ✅ 実装可能: Foldable (汎用定義)

```rust
pub trait Foldable<Op, Init> {
    type Output;
}

// Iterable経由のデフォルト実装
impl<T, Op, Init> Foldable<Op, Init> for T
where
    T: Iterable,
    <T as Iterable>::ToArray: FoldHelper<Op, Init>
{
    type Output = <<T as Iterable>::ToArray as FoldHelper<Op, Init>>::Output;
}
```

**問題点**: Rustのorphan ruleにより、blanket implは同一crateでのみ可能。
**解決策**: `Foldable`を直接各型に実装、またはマーカートレイト+`FoldHelper`委譲。

### ✅ 実装可能: Mappable (構造保持)

```rust
pub trait Mappable<F> {
    type Output; // 同じ構造を保持
}

// Array: 構造保持
impl<F, H, T: IsList + Mappable<F>> Mappable<F> for Array<H, T>
where F: Apply<H>, F::Output: Eval {
    type Output = Array<Evaluate<F::Output>, <T as Mappable<F>>::Output>;
}

// TreeArray: 構造保持 (値のみ変換)
impl<F, V, L, R> Mappable<F> for TreeArray<V, L, R>
where F: Apply<V>, L: Mappable<F>, R: Mappable<F> {
    type Output = TreeArray<Evaluate<F::Output>, <L as Mappable<F>>::Output, <R as Mappable<F>>::Output>;
}
```

**観察**: 構造を保持するMapは各型に個別実装が必要。

### ⚠️ 要検討: ConsLike / TreeLike

```rust
// ConsLike: Head/Tail/Cons 操作
pub trait ConsLike: IsList {
    type Head;
    type Tail: IsList;
    type Cons<NewHead>: IsList;
}

// 既存 List trait と統合可能
// 破壊的変更: List を ConsLike にリネーム、または統合
```

**判断**: 既存`List`トレイトを`ConsLike`に拡張するか、別トレイトとして定義するか選択が必要。

---

## Commonality Scope

### 完全共通化可能

| 操作                 | 方法                            |
| -------------------- | ------------------------------- |
| `Len`                | 各型に個別実装 (同一シグネチャ) |
| `IsEmpty`            | 各型に個別実装                  |
| `Fold`               | `Iterable::ToArray` 経由で委譲  |
| `ToArray` / `ToList` | 各型に個別実装                  |

### 構造依存 (個別実装必要)

| 操作             | 理由                                |
| ---------------- | ----------------------------------- |
| `Map`            | 構造保持が必要                      |
| `Filter`         | 結果構造が異なる（Tree → Array？）  |
| `Insert` / `Get` | 構造固有のセマンティクス            |
| `Contains`       | 探索戦略が異なる (O(n) vs O(log n)) |

### 変換経由で共通化可能

| 操作                   | 方法                                  |
| ---------------------- | ------------------------------------- |
| `Reverse`              | `ToArray` → `Reverse` → (構造再構築?) |
| `Take` / `Drop`        | `ToArray` 経由                        |
| `Find` / `Any` / `All` | `Fold` で実装可能                     |

---

## Proposed Trait Hierarchy

```
                    Eval (typelude-core)
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
    IsList           IsTreeArray       IsTreeMap
        │                │                │
        ▼                ▼                ▼
    ConsLike          TreeLike        TreeLike
   (Head/Tail/Cons)  (Value/L/R)    (K/V/L/R)
        │                │                │
        └───────┬────────┴────────────────┘
                ▼
            Iterable (ToArray)
                │
        ┌───────┼───────┐
        ▼       ▼       ▼
    Foldable Mappable Filterable
```

---

## Breaking Changes Required

1. **`List` → `ConsLike` リネーム** (または統合)
2. **`TreeArrayToList` / `TreeMapToList` → `Iterable::ToArray` に統一**
3. **`MapHelper` / `FoldHelper` / `FilterHelper` を汎用化**

---

## Verification Plan

### Automated Tests

```bash
# 全テスト実行
cargo test --workspace

# typelude-std のみ
cargo test -p typelude-std
```

### Test Coverage

1. `Iterable` 実装テスト: 各型で `ToArray` が正しく動作
2. `Foldable` テスト: `ToArray` 経由で `EFold` が動作
3. `Mappable` テスト: 構造が保持されること (TreeArray → TreeArray)
4. 互換性テスト: 既存コードが破壊されないこと

---

## Extensibility Analysis (RFC-0004 Compatibility)

### 評価対象: Advanced Collections

| 構造                | Iterable | Foldable     | Mappable | 備考           |
| ------------------- | -------- | ------------ | -------- | -------------- |
| `BitSet<Bits>`      | ⚠️        | ⚠️ PopCount的 | ❌        | O(1)操作が本質 |
| `Queue<In, Out>`    | ✅        | ✅            | ✅        | 構造保持可能   |
| `Trie<V, Children>` | ⚠️        | ⚠️            | ✅ 値のみ | 部分走査が本質 |
| `AVL<K,V,L,R,Bal>`  | ✅        | ✅            | ✅        | 再平衡必要     |
| `2-3 Tree`          | ✅        | ✅            | ✅        | 多分岐対応     |

### アプローチ比較

| 案      | 概要               | 実装コスト | 性能 | RFC-0004適合 |
| ------- | ------------------ | ---------- | ---- | ------------ |
| **案1** | `ToArray`中心      | ◎ 低       | △    | 60%          |
| **案2** | 操作別独立トレイト | △ 高       | ◎    | 85%          |
| **案3** | ハイブリッド       | ○ 中       | ○〜◎ | 90%          |

### 案3（推奨）: ハイブリッドアプローチ

```rust
// デフォルト: ToArray 経由
pub trait Iterable {
    type Item;
    type ToArray: IsList;
}

// 構造別最適化はオーバーライド
impl<Bits, Op, Init> Foldable<Op, Init> for BitSet<Bits>
where /* 直接 popcount 等を使用 */ {
    type Output = /* optimized */;
}
```

**利点:**
- デフォルト動作は即座に全構造カバー
- 必要に応じて最適化オーバーライド可能
- 段階的な最適化投資が可能

**構造別の最適化例:**
- `BitSet`: `Fold` を PopCount で O(1) 実装
- `Queue`: `Map` で `Queue<Map(In), Map(Out)>` 構造保持
- `Trie`: `Map` で値のみ変換、構造固有操作は維持
