# typelude-v2: メモ

## 基本的な原則

1. 厳密な型レベルプログラミング / rustcとの対話をやりやすくする
2. 拡張・応用可能である
3. 形式的・意味論的な妥当性を検証できる

## 重要なポイント

- Rustの型レベルプログラミングのメンタルモデルを明確にしないといけない。
  - impl + 関連型が分岐の本体である
    - そして分岐の副産物として再帰も可能である（ベースケースを作れる）
  - implにおいて露出する・連鎖するトレイト境界は何か
    - どうすればトレイト境界を管理しきれるか（UXに関わる）
    - おそらく入力そのものに対してで限られる？

- AST型の記述は維持する: `And<Lhf, Rhf>`や`If<Cond, Then, Else>`など
  - 流石に`<() as IfHelper<Cond, Then, Else>>::Output`を毎回書きたくは無いとされている
  - つまり: `struct And<Lhf, Rhf>`か`type And<Lhf, Rhf>`は定義することになる
    - 構造体 + 汎用トレイト
    - type alias

- 結局抽象をどう積み上げるか
  - 旧来の`Eval`, `Op*`, `E*`, `*Helper`含めて
    - `Eval`は多分削る
      - **嘘、高階関数に必要**
      - 例えば**While**なら、`Fn, Arg`を取りたいわけだが、AST的な構造体を取って`Generics<Generics>`は不可能である
        - `Op for (Arg)`
      - では`Apply<Arg> for Op`や`Apply<Op> for Arg`や`Apply for (Op, Arg)`や`Apply<Op, Arg> for ()`が欲しくなる
        - それ、結局薄い関数適用抽象が必要という話では？（Applyを噛ませて二重ジェネリクスで怒られないようにする）
        - それってEvalでは？
        - 実装はOp中心のほうが楽（とりあえずそのままWhileやMapの高階で使える）
          - それを強制するならEval<Op, Arg>のtraitか構造体で形を堅牢にしたほうがよいだろう
            - というこよでApply for (Op, Arg)は自由すぎるので却下
          - AST<Arg>と{Eval|Apply}<Op, Arg>を強制できるAPI/型定義
    - 今の`Op*`+`E*`のような、抽象操作のトレイト + AST構成は残すかも
      - 対応・拡張がしやすい
  - ただ、固定できるのはある
    - ほしいAST
      - `And<Lhf, Rhf>`, `Or<Lhf, Rhf>`, `Not<Arg>`, `If<Cond, Then, Else>`など
    - ほしい機能・分岐（本質的なimplに求められる分岐）
    - プリミティブなAtom（`True`, `False`, `TArr`, `Tail`など）
  - ASTと分岐実体とAtomさえ固定できていれば、抽象の積み上げ方は慎重にやれば割とどうにでもなるし、どういうやり方もできる
    - 例えばBoolであれば
      - NandHelper基盤: 真理値表の定義

        ```rust
        impl NandHelper<True, True> for () { type Output = False; }
        impl NandHelper<False, True> for () { type Output = True; }
        impl NandHelper<True, False> for () { type Output = True; }
        impl NandHelper<False, False> for () { type Output = True; }
        ```

      - IsBoolの関連型基盤

        ```rust
        trait IsBool {
            const VALUE: bool;

            type Not: IsBool;
            type Nand<B: IsBool>: IsBool;
            type And<B: IsBool>: IsBool;
            type Or<B: IsBool>: IsBool;
            type Xor<B: IsBool>: IsBool;
        }
        ```

      - どっちを使うせよ、`Nand<A, B>`と書きたいなら抽象を積むことになる
        - 関連型なら

          ```rust
          type Nand<A: IsBool, B: IsBool> = <() as NandHelper<A, B>>::Output;
          ```

          ```rust
          type Nand<A: IsBool, B: IsBool> = <A as IsBool>::Nand<B>;
          ```

        - `OpNand`を積むなら

          ```rust
          impl<A: IsBool, B: IsBool> OpNand for (A, B) {
              type Output = <() as NandHelper<A, B>>::Output;
              type PrimOutput = <A as IsBool>::Nand<B>;
          }
          ```

- クレート分割について
  - 方針
    - 拡張性・疎結合（アーキテクチャの堅牢化）・少しずつクレートを入れられるようにする（サイズを小さくする）あたり
  - クレート一覧
    - `typelude`
    - `typelude-core` / `typelude-std`
      - 制御系
        - If
        - While
        - Eq / NEq
      - 基本の共通インターフェース・トレイトの設計
        - Add, Sub, Mul, Div, Rem, Neg, Not, Shl, Shr, BitAnd, BitOr, BitXorなどの基本演算子の共通インターフェース
          - おそらくトレイト + Typealias
        - From, Into, Reify
        - Option, Result
        - Debug / Displayなど汎用ユーティリティ系（後述）
      - デバッグ・テストエコシステム系
        - `static_assertions`系のやつ
        -
      - 型プログラミング汎用ユーティリティ
        - 型レベルOptionもそう
        -
    - `typelude-bool`
    - `typelude-col`
    - `typelude-std`
    - `typelude-num`
      - `typenum`ベース
    - `typelude-`

- エコシステム含めた機能
  - 実際のユースケースの想定とそれを踏まえた機能の増強
    - typenumを参考に仕様。
    - **値/ランタイムの世界との接続**
    - **Constとの統合運用**
      - Constができないことを一覧化すうる
    - **他のクレートとの相互運用**
      - このためには拡張性、あるいは耐えうるAPI/抽象/型設計が必要である
      - 型レベル
      - typenum
      - generic-array
      - konst
      - generic programming
        - frunk
        - typewit
        - type-const
        - tstr
      - compile-time
        - const_format_crates
        - const_format
        - const_panic
        - static_assertions
        - build_assert
        - seq-macro
      - 型レベル可変
        - heapless
        - hybrid-array
        - nalgebra
        - generic_array_storage
        - uom
        - dimensioned
  - std / rustcの関連機能wrap + 補助
    - `std::any::type_name`
      - これはかなり使える予感がしませんか？
    - `std::any::TypeId`
    - `std::any::Any`
    - `std::mem::size_of`
    - `std::mem::align_of`
    - `std::marker::PhantomData`
      - Wrapして色々使えるかもしれん

      - ```rust
        struct TypeTag<T>(PhantomData<T>);

        impl<T> Debug for TypeTag<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                write!(f, "TypeTag<{}>", std::any::type_name::<T>())
            }
        }
        ```

  - デバッグ / テスト
    - この全体含むが
    - `static_assertion`系のやつもだし
    - Debug / Diplayもだし
    - Trace
  - 記述支援もそう
  - Debug / Displayトレイト系
    - ただ型って色々見せ方したいので悩む！
      - いろんな構造で見せたいことがある
      - 型構造そのまま見せたいこともあるし
      - VM系はスタックの状態を見せたいこともあるし
    - `DisplayHelper<Mode, T>`みたいにしてもいいかもしれない。いや、わからない。
  - inspect / profiler
    - `tooling`くんです
