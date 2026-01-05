---
id: rfc-0004
title: "Advanced Type-Level Collections"
status: draft
created: 2026-01-06
author:
---

# RFC-0004: Advanced Type-Level Collections

## Summary

Introduce advanced type-level data structures beyond the existing `Array`, `Map`, `TreeArray`, and `TreeMap`. This RFC covers structures offering O(1) performance (`BitSet`), amortized O(1) queues (`Queue`, `Deque`), named field access (`TupleRecord`), and experimental hash-based structures (`HashMap`, `Trie`).

## Motivation

### Current State (RFC-0001 Completed)

| Structure             | Type Signature | Lookup    | Use Case           |
| :-------------------- | :------------- | :-------- | :----------------- |
| `Array<H, T>`         | Cons List      | O(N)      | Sequential data    |
| `Map<K, V, T>`        | Assoc List     | O(N)      | K-V pairs (linear) |
| `TreeArray<V, L, R>`  | BST            | O(log N)* | Sorted set         |
| `TreeMap<K, V, L, R>` | BST            | O(log N)* | Sorted map         |

*Average case; worst case O(N) for unbalanced trees.

### Limitations & Opportunities

| Problem                   | Proposed Solution                       |
| :------------------------ | :-------------------------------------- |
| No O(1) set operations    | `BitSet` using `typenum` integers       |
| Queue operations are O(N) | Double-stack functional `Queue`/`Deque` |
| Positional-only access    | `TupleRecord` with named fields         |
| No hash-based lookups     | `HashMap` with const fn hashing         |
| String key lookup is O(N) | `Trie` for prefix-based access          |

---

## Detailed Design

### Part 1: `BitSet` — O(1) Type-Level Set

Leverage `typenum` unsigned integers as bitmasks. Each bit position represents set membership.

#### Data Structure

```rust
use typenum::{Unsigned, U0};
use std::marker::PhantomData;

/// Type-level BitSet wrapper
pub struct BitSet<Bits: Unsigned>(PhantomData<Bits>);

/// Empty set
pub type EmptyBitSet = BitSet<U0>;
```

#### Operations

```rust
use typenum::{Or, And, Shl, B1, IsEqual};

/// Insert element at index I
pub trait BitSetInsert<I: Unsigned> {
    type Output;
}

impl<Bits: Unsigned, I: Unsigned> BitSetInsert<I> for BitSet<Bits>
where
    B1: Shl<I>,                           // 1 << I
    <B1 as Shl<I>>::Output: Unsigned,
    Bits: Or<<B1 as Shl<I>>::Output>,     // bits | (1 << I)
{
    type Output = BitSet<<Bits as Or<<B1 as Shl<I>>::Output>>::Output>;
}

/// Check if element at index I is present
pub trait BitSetContains<I: Unsigned> {
    type Output; // True or False
}

/// Union of two BitSets
pub trait BitSetUnion<Other> {
    type Output;
}

impl<A: Unsigned, B: Unsigned> BitSetUnion<BitSet<B>> for BitSet<A>
where
    A: Or<B>,
{
    type Output = BitSet<<A as Or<B>>::Output>;
}

/// Intersection of two BitSets
pub trait BitSetIntersection<Other> {
    type Output;
}

impl<A: Unsigned, B: Unsigned> BitSetIntersection<BitSet<B>> for BitSet<A>
where
    A: And<B>,
{
    type Output = BitSet<<A as And<B>>::Output>;
}
```

#### Performance

| Operation      | Complexity | Notes                        |
| :------------- | :--------- | :--------------------------- |
| `Insert`       | O(1)       | Single bitwise OR            |
| `Contains`     | O(1)       | Single bitwise AND + compare |
| `Union`        | O(1)       | Single bitwise OR            |
| `Intersection` | O(1)       | Single bitwise AND           |

#### Limitations

- Maximum ~128 elements (limited by typenum's largest integers)
- Elements must be mapped to indices externally

---

### Part 2: `Queue` / `Deque` — Amortized O(1) FIFO

Functional queue using two stacks (banker's queue).

#### Data Structure

```rust
use crate::data::col::array::{Array, Nil, IsList};

/// Double-stack queue
/// - `In`: Elements pushed (reversed order)
/// - `Out`: Elements to pop (correct order)
pub struct Queue<In: IsList, Out: IsList>(PhantomData<(In, Out)>);

/// Empty queue
pub type EmptyQueue = Queue<Nil, Nil>;

/// Double-ended queue (same structure, different ops)
pub struct Deque<Front: IsList, Back: IsList>(PhantomData<(Front, Back)>);
```

#### Operations

```rust
/// Enqueue: Add to back
pub trait Enqueue<V> {
    type Output;
}

impl<V, In: IsList, Out: IsList> Enqueue<V> for Queue<In, Out> {
    // Prepend to In stack
    type Output = Queue<Array<V, In>, Out>;
}

/// Dequeue: Remove from front
pub trait Dequeue {
    type Value;   // Head element
    type Rest;    // Remaining queue
}

// Case 1: Out is non-empty → pop from Out
impl<V, OutTail: IsList, In: IsList> Dequeue for Queue<In, Array<V, OutTail>> {
    type Value = V;
    type Rest = Queue<In, OutTail>;
}

// Case 2: Out is empty → reverse In to Out, then pop
impl<V, InTail: IsList> Dequeue for Queue<Array<V, InTail>, Nil>
where
    Array<V, InTail>: Reverse,
    <Array<V, InTail> as Reverse>::Output: Dequeue,
{
    type Value = <<Array<V, InTail> as Reverse>::Output as Dequeue>::Value;
    type Rest = <<Array<V, InTail> as Reverse>::Output as Dequeue>::Rest;
}
```

#### Performance

| Operation | Amortized | Worst Case     |
| :-------- | :-------- | :------------- |
| `Enqueue` | O(1)      | O(1)           |
| `Dequeue` | O(1)      | O(N) (reverse) |

---

### Part 3: `TupleRecord` — Named Field Access

HList-based structure with type-level field names.

#### Data Structure

```rust
/// Field marker (compile-time only)
pub struct Field<Name, Value>(PhantomData<(Name, Value)>);

/// Record is an HList of Fields
pub type PersonRecord = (
    Field<Name, String>,
    Field<Age, u32>,
    Field<Email, String>,
);

// Or using a macro:
// record!{ name: String, age: u32, email: String }
```

#### Operations

```rust
/// Get field by name
pub trait RecordGet<FieldName> {
    type Output;
}

/// Set field by name
pub trait RecordSet<FieldName, NewValue> {
    type Output;
}

/// Check if field exists
pub trait RecordHas<FieldName> {
    type Output; // True or False
}

/// Merge two records (right overrides left)
pub trait RecordMerge<Other> {
    type Output;
}
```

#### Example Usage

```rust
// Define field name types
struct Name;
struct Age;

// Create record
type Person = Record![(Name, String), (Age, u32)];

// Access
type PersonName = <Person as RecordGet<Name>>::Output; // String
type PersonAge = <Person as RecordGet<Age>>::Output;   // u32

// Update
type OlderPerson = <Person as RecordSet<Age, u64>>::Output;
```

---

### Part 4: `HashMap` — Const-fn Hybrid Approach

Use `const fn` to compute hash values at compile time, then use `typenum` for bucket indexing.

#### Hashing Strategy

**Option A: SipHash-like (Lightweight)**

```rust
const fn siphash_2_4(key: &[u8]) -> u64 {
    // Simplified SipHash: 2 compression rounds, 4 finalization rounds
    let mut v0 = 0x736f6d6570736575u64;
    let mut v1 = 0x646f72616e646f6du64;
    // ... ARX operations ...
    v0 ^ v1
}
```

**Option B: Simple Polynomial Hash**

```rust
const fn poly_hash(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut h = 0u64;
    let mut i = 0;
    while i < bytes.len() {
        h = h.wrapping_mul(31).wrapping_add(bytes[i] as u64);
        i += 1;
    }
    h
}
```

#### Type-Level Integration

```rust
/// Trait for types that can be hashed at compile time
pub trait TypeHash {
    type Hash: Unsigned;
}

/// String literal hashing (requires const generics)
impl<const S: &'static str> TypeHash for TyStr<S> {
    type Hash = /* const { poly_hash(S) } as typenum */;
}

/// Bucket access via modulo
type BucketIndex<K, N> = <<K as TypeHash>::Hash as Rem<N>>::Output;
```

#### Data Structure

```rust
/// Fixed-size bucket array
pub struct HashMap<const BUCKET_COUNT: usize, Buckets>(PhantomData<Buckets>);

/// Each bucket is a list of (K, V) pairs (for collision handling)
type Bucket<K, V> = Array<(K, V), Nil>;
```

#### Challenges

| Challenge                         | Mitigation                                          |
| :-------------------------------- | :-------------------------------------------------- |
| `const fn` → `typenum` conversion | Use `generic_const_exprs` (nightly) or macro bridge |
| Collision handling                | Linear chaining within bucket                       |
| Limited key types                 | Only types implementing `TypeHash`                  |

---

### Part 5: `Trie` — Prefix Tree for Strings

Optimized for string keys using type-level character branching.

#### Data Structure

```rust
/// Trie node
pub struct TrieNode<Value, Children>(PhantomData<(Value, Children)>);
// Value: Option-like (Some<V> or None)
// Children: Map from char to TrieNode

/// Empty trie
pub struct TrieNil;
```

#### Operations

```rust
/// Insert key-value pair
pub trait TrieInsert<Key, Value> {
    type Output;
}

/// Get value by key
pub trait TrieGet<Key> {
    type Output; // Some<V> or None
}

/// Longest prefix match
pub trait TrieLongestPrefix<Key> {
    type MatchedKey;
    type Value;
}
```

#### Implementation Notes

- Key representation: HList of characters or type-level string
- Children map: Use `TreeMap<Char, TrieNode>` for O(log alphabet) branching
- Performance: O(L) where L = key length

---

### Part 6: `AVL Tree` — Self-Balancing BST

Extension of `TreeArray`/`TreeMap` with automatic rebalancing.

#### Data Structure

```rust
use typenum::{Integer, Z0, P1, N1};

/// AVL node with balance factor
pub struct AvlNode<K, V, Left, Right, Balance: Integer>(
    PhantomData<(K, V, Left, Right, Balance)>
);

// Balance ∈ {N1, Z0, P1} = {-1, 0, +1}
```

#### Rotation Operations

```rust
/// Right rotation (for left-heavy trees)
pub trait RotateRight {
    type Output;
}

/// Left rotation (for right-heavy trees)
pub trait RotateLeft {
    type Output;
}

/// Double rotation: Left-Right
pub trait RotateLR {
    type Output;
}

/// Double rotation: Right-Left
pub trait RotateRL {
    type Output;
}
```

#### Rebalancing Logic

| Balance Factor   | Condition               | Action         |
| :--------------- | :---------------------- | :------------- |
| -2 (left-heavy)  | Left child balance ≤ 0  | Right rotation |
| -2               | Left child balance > 0  | LR rotation    |
| +2 (right-heavy) | Right child balance ≥ 0 | Left rotation  |
| +2               | Right child balance < 0 | RL rotation    |

---

### Part 7: `2-3 Tree` — Multi-way Balanced Tree

Simpler alternative to AVL with guaranteed balance.

#### Data Structure

```rust
pub enum Node23<K, V> {
    Nil,
    Two {
        k1: K, v1: V,
        left: Node23<K, V>,
        right: Node23<K, V>,
    },
    Three {
        k1: K, v1: V,
        k2: K, v2: V,
        left: Node23<K, V>,
        middle: Node23<K, V>,
        right: Node23<K, V>,
    },
}

// Type-level equivalent:
pub struct Two<K1, V1, Left, Right>(PhantomData<(K1, V1, Left, Right)>);
pub struct Three<K1, V1, K2, V2, L, M, R>(PhantomData<(K1, V1, K2, V2, L, M, R)>);
```

#### Benefits

- Always perfectly balanced
- Shallower than binary trees → less recursion depth
- Insertion splits nodes instead of rotating

---

## Alternatives Considered

| Alternative               | Pros                    | Cons                          |
| :------------------------ | :---------------------- | :---------------------------- |
| Runtime-only collections  | Simpler                 | Loses compile-time guarantees |
| Macro-heavy generation    | Avoids trait complexity | Code bloat, harder to debug   |
| External proc-macro crate | More flexibility        | Additional dependency         |

---

## Unresolved Questions

1. **typenum capacity**: Is 128-bit sufficient for `BitSet`? Consider chunked `BitSet256`.
2. **const fn stability**: When will `generic_const_exprs` stabilize?
3. **String representation**: `const &str` vs HList of chars for `Trie` keys?
4. **Naming**: `TupleRecord` vs `HRecord` vs `TypeRecord`?

---

## Implementation Phases

| Phase | Structure         | Effort | Priority | Dependencies       |
| :---- | :---------------- | :----- | :------- | :----------------- |
| 1     | `BitSet`          | Low    | High     | None               |
| 2     | `Queue` / `Deque` | Low    | High     | `Reverse` op       |
| 3     | `TupleRecord`     | Medium | High     | HList pattern      |
| 4     | `AVL Tree`        | High   | Medium   | Existing `TreeMap` |
| 5     | `Trie`            | High   | Medium   | Type-level strings |
| 6     | `HashMap`         | High   | Low      | Const fn + nightly |
| 7     | `2-3 Tree`        | Medium | Low      | None               |

---

## References

- [typenum docs](https://docs.rs/typenum/latest/typenum/) — Compile-time numerics
- [frunk HList](https://docs.rs/frunk/latest/frunk/) — Heterogeneous lists in Rust
- [Banker's Queue](https://en.wikipedia.org/wiki/Queue_(abstract_data_type)#Purely_functional_implementation) — Amortized O(1) queue
- [Okasaki's Purely Functional Data Structures](https://www.cs.cmu.edu/~rwh/theses/okasaki.pdf) — Theoretical foundation
