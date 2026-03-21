# Philosophy and Theory

typeludeの設計哲学と理論的背景。

## 基本的な立場

Rustの型システムは**論理プログラミングシステム**として動作する。

- `impl Trait for Type` → Horn節（推論規則）
- 型検査 → Prolog的な証明探索
- 関連型 → 型レベル関数の出力

この性質から、型システムは「計算を行う場所」として使える。
「コンパイルが通る」こと自体が、型レベルでの計算結果の正しさの証明になる（Curry-Howard対応）。

## 型システムの表現力

### 型システムの階層

```
System T    原始再帰のみ         → 強正規化（必ず停止）
    ↓
System F    多相型（∀）          → 強正規化、Turing完全でない
    ↓
System Fω   型演算子（型 → 型）  → まだ正規化可能
    ↓
Fω + μ      不動点を追加         → Turing完全（停止保証なし）
```

Rustのtraitシステムは実質 **System Fω + μ** に相当する。

| Rustの機能 | 理論的対応 |
| --- | --- |
| Generics (`<T>`) | 全称量化 ∀ (System F) |
| 関連型 (`type Output`) | 型演算子 (System Fω) |
| GATs / `TyFn<Arg>` | 高階型（型 → 型の関数） |
| recursive trait bounds | 不動点演算子 μ |
| distinct impl dispatch | 型による場合分け（条件分岐） |

### Turing完全性の根拠

`EWhile` の実装は型レベルの不動点演算子（μ演算子）：

```
while(pred, step, s) = μF. λs. if pred(s) then F(step(s)) else s
```

recursive trait boundsがこのμ演算子を与える。
`recursion_limit = "65536"` は理論上の無限再帰を実用上の有限に制限しているだけ。

Haskellの文脈では「型クラス + UndecidableInstances = Turing完全」として知られており
（Mark P. Jones, "Type Classes with Functional Dependencies", ESOP 2000）、
typeludeのEvalパターンはその構造的等価物。

## 計算の十分条件

型レベルで「プログラム」として十分な表現力を持つための3要素：

1. **条件分岐** — distinct implのdispatch（`True`と`False`に別々のimpl）
2. **再帰 / 反復** — recursive trait bounds（μ演算子として機能）
3. **状態** — 型引数による状態の伝播（関連型が「戻り値」になる）

この3つが揃うことでSystem FωにμとConditionが加わり、Turing完全になる。

## Evalパターンの理論的位置づけ

Evalパターンは型理論の構造と対応している：

| Evalの要素 | 理論的対応 |
| --- | --- |
| `trait Eval { type Output; }` | 型レベル関数の適用規則 |
| `impl Eval for EFoo` 内への制約封じ込め | System Fω における型演算子の抽象化 |
| `ELit<T>` | η変換（値の持ち上げ） |
| `EApp<F, A>` | β簡約（関数適用） |
| `TyFn<Arg>` | 高階型（型 → 型の関数） |

型エイリアスが「透明」（制約が呼び出し側に漏れる）なのに対し、
Evalパターンは「不透明」（制約をimpl内に封じ込める）。
この違いが「制約の爆発を防ぐ」という実用上の動機の理論的根拠。

Richard Eisenbergの "Dependent Types in Haskell: Theory and Practice" (2016) では
Haskellの型族（type families）がSystem Fωの表現力を持つことを形式化している。
RustのEvalパターンはこの表現力をtraitシステムの上に構築したものと見なせる。

## 参考文献

- Mark P. Jones, "Type Classes with Functional Dependencies", ESOP 2000
- Richard A. Eisenberg, "Dependent Types in Haskell: Theory and Practice", 2016 (arXiv:1610.07978)
- J. Garrett Morris & Richard Eisenberg, "Constrained Type Families", 2017 (arXiv:1706.09715)
- Benjamin C. Pierce, "Types and Programming Languages" (TAPL)
- sdleffler, "Rust's Type System is Turing-Complete" https://sdleffler.github.io/RustTypeSystemTuringComplete/
- Will Crichton, "Type-level Programming in Rust" https://willcrichton.net/notes/type-level-programming/
