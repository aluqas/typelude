---
id: rfc-0001
title: "Collections Extension: Map, Tree & Comprehensive Operations"
status: draft
created: 2026-01-05
author:
---

# RFC-0001: Collections Extension

## Summary

Extend typelude's collections module with new data structures (`TyMap`, `TyTree`) and comprehensive operations, enabling O(log N) compile-time access and richer type-level data manipulation.

## Motivation

### Current State

The existing `TyArray` (Cons list) provides:
- Linear data storage with O(N) random access
- Basic operations: `Head`, `Tail`, `Get`, `Set`, `Concat`
- Higher-order functions: `Map`, `Filter`, `Fold`

### Limitations

1. **Performance**: `Get<N>` requires N trait resolutions - problematic for large lists
2. **Key-Value Storage**: No native associative container support
3. **Missing Operations**: Common functional operations like `Zip`, `Reverse`, `Find` are absent

### Goals

1. Introduce `TyMap<Key, Value>` for associative storage
2. Introduce `TyTree` for O(log N) balanced access
3. Systematically evaluate and implement missing operations

---

## Detailed Design

### Part 1: Operation Catalog & Evaluation

Comprehensive evaluation of all potential operations across dimensions:

| Rating | Feasibility          | Complexity      | Utility     | Performance        |
| ------ | -------------------- | --------------- | ----------- | ------------------ |
| ◎      | Straightforward impl | Low boilerplate | High demand | O(N) or better     |
| ○      | Requires care        | Moderate        | Useful      | O(N²) acceptable   |
| △      | Experimental         | High            | Niche       | Compile-time heavy |
| ✗      | Blocked by Rust      | —               | —           | —                  |

---

#### 1.1 Existing Operations (Implemented)

| Operation        | Feasibility | Complexity | Utility | Performance | Status      |
| ---------------- | ----------- | ---------- | ------- | ----------- | ----------- |
| `Len`            | ◎           | ◎          | ◎       | O(N)        | ✅           |
| `Head`           | ◎           | ◎          | ◎       | O(1)        | ✅           |
| `Tail`           | ◎           | ◎          | ◎       | O(1)        | ✅           |
| `IsEmpty`        | ◎           | ◎          | ◎       | O(1)        | ✅           |
| `Get<Idx>`       | ◎           | ◎          | ◎       | O(N)        | ✅           |
| `Set<Idx, Val>`  | ◎           | ○          | ◎       | O(N)        | ✅           |
| `Concat`         | ◎           | ◎          | ◎       | O(N)        | ✅           |
| `Append`         | ◎           | ◎          | ◎       | O(N)        | ✅           |
| `Prepend`        | ◎           | ◎          | ◎       | O(1)        | ✅           |
| `Contains`       | ◎           | ○          | ◎       | O(N)        | ✅ (nightly) |
| `Map<Op>`        | ◎           | ○          | ◎       | O(N)        | ✅           |
| `Filter<Pred>`   | ○           | ○          | ◎       | O(N)        | ✅           |
| `Fold<Op, Init>` | ◎           | ○          | ◎       | O(N)        | ✅           |

---

#### 1.2 Proposed New Operations

##### High Priority (◎ Utility)

| Operation    | Feasibility | Complexity | Utility | Performance | Notes                  |
| ------------ | ----------- | ---------- | ------- | ----------- | ---------------------- |
| `Reverse`    | ◎           | ◎          | ◎       | O(N)        | Fold-based impl        |
| `Take<N>`    | ◎           | ◎          | ◎       | O(N)        | Recursive descent      |
| `Drop<N>`    | ◎           | ◎          | ◎       | O(N)        | Recursive descent      |
| `Zip<Other>` | ◎           | ○          | ◎       | O(N)        | Parallel recursion     |
| `Flatten`    | ◎           | ○          | ◎       | O(N×M)      | Nested list → flat     |
| `Find<Pred>` | ○           | ○          | ◎       | O(N)        | Returns `Option`-like  |
| `Any<Pred>`  | ◎           | ◎          | ◎       | O(N)        | Short-circuit via Fold |
| `All<Pred>`  | ◎           | ◎          | ◎       | O(N)        | Short-circuit via Fold |

##### Medium Priority (○ Utility)

| Operation           | Feasibility | Complexity | Utility | Performance | Notes                     |
| ------------------- | ----------- | ---------- | ------- | ----------- | ------------------------- |
| `FlatMap<Op>`       | ◎           | ○          | ○       | O(N×M)      | Map + Flatten combo       |
| `Partition<Pred>`   | ○           | ○          | ○       | O(N)        | Returns `(Matches, Rest)` |
| `TakeWhile<Pred>`   | ○           | ○          | ○       | O(N)        | Prefix matching           |
| `DropWhile<Pred>`   | ○           | ○          | ○       | O(N)        | Skip prefix               |
| `Slice<Start, End>` | ○           | ○          | ○       | O(N)        | `Take . Drop` combo       |
| `IndexOf<Elem>`     | ○           | ○          | ○       | O(N)        | Returns index or None     |
| `Count<Pred>`       | ◎           | ◎          | ○       | O(N)        | Fold-based                |
| `Intersperse<Sep>`  | ◎           | ○          | ○       | O(N)        | Insert between elems      |
| `Last`              | ◎           | ◎          | ○       | O(N)        | Fold to final elem        |
| `Init`              | ○           | ○          | ○       | O(N)        | All except last           |

##### Low Priority / Experimental (△)

| Operation         | Feasibility | Complexity | Utility | Performance | Notes                              |
| ----------------- | ----------- | ---------- | ------- | ----------- | ---------------------------------- |
| `Sort<Cmp>`       | △           | △          | △       | O(N² log N) | Type-level mergesort; very heavy   |
| `Unique`          | △           | △          | ○       | O(N²)       | Requires equality across all pairs |
| `GroupBy<Key>`    | △           | △          | △       | O(N²)       | Returns nested lists               |
| `Permutations`    | ✗           | ✗          | △       | O(N!)       | Exponential; impractical           |
| `Combinations<K>` | △           | △          | △       | O(N^K)      | Limited K only                     |

---

### Part 2: New Data Structures

#### 2.1 `TyMap<K, V>` — Type-Level Associative Map

```rust
// Data Structure (List of Pairs)
pub struct TyMapNil;
pub struct TyMapCons<Key, Value, Tail>(PhantomData<(Key, Value, Tail)>);

// Core Trait
pub trait TypeMap {
    type Get<K>: MaybeType;       // Returns TySome<V> or TyNone
    type Insert<K, V>: TypeMap;   // Upsert semantics
    type Remove<K>: TypeMap;
    type Keys: Cons;              // Extract all keys
    type Values: Cons;            // Extract all values
}
```

| Operation      | Feasibility | Complexity | Utility | Performance | Notes                 |
| -------------- | ----------- | ---------- | ------- | ----------- | --------------------- |
| `Get<K>`       | ◎           | ○          | ◎       | O(N)        | Linear search on keys |
| `Insert<K, V>` | ◎           | ○          | ◎       | O(N)        | Check + prepend       |
| `Remove<K>`    | ○           | ○          | ○       | O(N)        | Filter-based          |
| `Contains<K>`  | ◎           | ◎          | ◎       | O(N)        | Key existence         |
| `Keys`         | ◎           | ◎          | ◎       | O(N)        | Map over pairs        |
| `Values`       | ◎           | ◎          | ◎       | O(N)        | Map over pairs        |
| `Merge<Other>` | ○           | ○          | ○       | O(N×M)      | Union with override   |

**Implementation Strategy:**
- Flat list of `(Key, Value)` pairs
- Linear search acceptable for typical < 100 entries at compile time
- Consider `TySortedMap` variant with binary-search (requires `Ord` on keys)

---

#### 2.2 `TyTree<T>` — Type-Level Balanced Tree

```rust
// AVL-style balanced binary tree
pub struct TyLeaf;
pub struct TyNode<Value, Left, Right, Height>(
    PhantomData<(Value, Left, Right, Height)>
);

// Core Trait
pub trait TypeTree {
    type Insert<V, Cmp>: TypeTree;  // Balanced insert
    type Contains<V, Cmp>: TyBool;  // O(log N) lookup
    type ToList: Cons;              // In-order traversal
    type Height: Unsigned;
}
```

| Operation     | Feasibility | Complexity | Utility | Performance | Notes                   |
| ------------- | ----------- | ---------- | ------- | ----------- | ----------------------- |
| `Insert<V>`   | △           | △          | ○       | O(log N)    | Requires rotation logic |
| `Contains<V>` | ○           | ○          | ◎       | O(log N)    | Binary search           |
| `Delete<V>`   | △           | △          | ○       | O(log N)    | Complex rebalancing     |
| `Min` / `Max` | ◎           | ◎          | ○       | O(log N)    | Leftmost/rightmost      |
| `ToList`      | ◎           | ○          | ○       | O(N)        | In-order traversal      |
| `FromList`    | △           | △          | ○       | O(N log N)  | Repeated insert         |

**Implementation Challenges:**
- Rotation logic: 4 cases (LL, LR, RL, RR) each need separate impl
- Height tracking: `typenum` arithmetic for balance factor
- Comparator trait: Need generic `TyOrd<A, B>` constraint

**Recommendation:** Start with unbalanced `TyBST`, graduate to AVL if performance justifies.

---

#### 2.3 Alternative: `TyVec` with Chunking

For truly large collections (100+ elements), consider:

```rust
// 16-element chunks for O(N/16) top-level + O(16) inner
pub struct TyVec<Chunks: Cons>(PhantomData<Chunks>);
// Each chunk is TyArray with max 16 elements
```

| Pro                                | Con                              |
| ---------------------------------- | -------------------------------- |
| Faster random access               | More complex impl                |
| Cache-friendly in trait resolution | Requires chunk boundary handling |

---

### Part 3: Module Organization

Proposed structure for `std/cols/`:

```
std/cols/
├── mod.rs           # Re-exports
├── array.rs         # Move from primitives/ (operations only)
├── map.rs           # TyMap implementation
├── tree.rs          # TyTree / TyBST implementation
├── ops/
│   ├── mod.rs
│   ├── transform.rs # Reverse, Flatten, Intersperse
│   ├── search.rs    # Find, IndexOf, Contains
│   ├── slice.rs     # Take, Drop, Slice
│   └── combine.rs   # Zip, Partition
└── traits.rs        # Shared capability traits
```

---

## Alternatives Considered

### 1. Keep Everything in `array.rs`
- **Pro**: Simpler discovery
- **Con**: File already 676 lines; will become unmanageable

### 2. Use `typenum` BitSets for Map Keys
- **Pro**: O(1) lookup for small integer keys
- **Con**: Limited key types; not generalizable

### 3. Implement via Macros Instead of Traits
- **Pro**: Simpler mental model
- **Con**: Loses composability with `Eval` pattern

---

## Unresolved Questions

1. **Naming**: `TyMap` vs `TypeMap` vs `HMap` (heterogeneous map)?
2. **Error Handling**: How to represent `None` / `KeyNotFound` at type level?
   - Option A: `TySome<V>` / `TyNone` wrapper
   - Option B: Compile error via missing impl
3. **Ordering**: Require `TyOrd` for `TyTree` or provide comparator as parameter?
4. **Migration**: Move `array.rs` operations to `cols/` now or later?

---

## Implementation Phases

| Phase | Scope                                                                 | Priority |
| ----- | --------------------------------------------------------------------- | -------- |
| **1** | New operations on `TyArray`: `Reverse`, `Take`, `Drop`, `Zip`, `Find` | High     |
| **2** | `TyMap` with basic `Get`/`Insert`/`Remove`                            | High     |
| **3** | Remaining array ops: `Flatten`, `Partition`, `TakeWhile`              | Medium   |
| **4** | `TyTree` (unbalanced BST first)                                       | Medium   |
| **5** | Balanced AVL tree variant                                             | Low      |

---

## References

- [frunk HList](https://docs.rs/frunk/latest/frunk/) — Rust HList patterns
- [typenum](https://docs.rs/typenum/latest/typenum/) — Type-level numerics
- [Purely Functional Data Structures (Okasaki)](https://www.cs.cmu.edu/~rwh/theses/okasaki.pdf) — Theoretical foundation
