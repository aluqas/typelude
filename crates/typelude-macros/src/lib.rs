//! Typelude 用プロシージャルマクロ群。
//!
//! 型レベル表現やDSL を Rust の構文として記述しやすくするための
//! マクロを提供。詳細な型制約やAST 展開を隠蔽し、ユーザーは
//! 型メタプログラミングに集中できる。
//!
//! ## 主要マクロ
//!
//! - `ty!`: 型レベル型式を記述
//! - `bound!`: 型制約を記述
//! - `impl_eval!`: Eval実装を素早く生成
//! - `ty_fn!`: 型レベル関数とEval実装を併せて生成
//! - `ty_expr!`: 型式と Eval 実装を展開
//! - `program!`: スタックマシンプログラムをコンパイル
//! - `twat!`: WebAssembly AST を型式へ変換

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod dsl;
mod program;
mod twat;
mod ty_fn_impl;

use dsl::{BoundDslInput, ImplEvalInput, TyDslInput};
use program::ProgramInput;
use twat::WasmWatInput;
use ty_fn_impl::TyFnInput;

/// 型レベル型式記述マクロ。
///
/// Typelude の型式をRust 的な構文で直接記述できるようにする。
/// 内部で式のパースと展開を行る。
///
/// # 入力フォーマット
///
/// ```text
/// ty! { /* 型式 */ }
/// ```
///
/// # 例
///
/// ```ignore
/// use typelude::ty;
///
/// // 型式をそのまま参照
/// type MyExpr = ty! { Add<3, 5> };
/// ```
#[proc_macro]
pub fn ty(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TyDslInput);
    let ty = input.ty;
    TokenStream::from(quote! { #ty })
}

/// 型制約記述マクロ。
///
/// 型パラメータに課す制約（where句等）を型式として記述する。
///
/// # 入力フォーマット
///
/// ```text
/// bound! { /* 制約式 */ }
/// ```
#[proc_macro]
pub fn bound(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as BoundDslInput);
    let bounds = input.bounds;
    TokenStream::from(quote! { #bounds })
}

/// Eval トレイト実装自動生成マクロ。
///
/// 型式から Output型への Eval 実装を素早く生成する。
///
/// # 入力フォーマット
///
/// ```text
/// impl_eval! {
///     target_type: /* 実装対象の型 */,
///     output_type: /* Output型 */,
///     generics: [/* 汎用パラメータ */],
///     where_clause: [/* 制約 */],  // optional
/// }
/// ```
///
/// # 生成結果
///
/// ```text
/// impl<...> typelude::core::Eval for TargetType where ... {
///     type Output = OutputType;
/// }
/// ```
#[proc_macro]
pub fn impl_eval(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ImplEvalInput);

    let generics = input.generics;
    let target_type = input.target_type;
    let output_type = input.output_type;

    let where_clause = if let Some(bounds) = input.where_clause {
        quote! { where #bounds }
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl #generics typelude::core::Eval for #target_type
        #where_clause
        {
            type Output = #output_type;
        }
    };

    TokenStream::from(expanded)
}

/// 型レベル関数定義マクロ。
///
/// 型パラメータをとる関数型と、その Eval 実装を同時に用意する。
/// 単なる型エイリアスではなく、制約内部化により再帰型対応を実現。
///
/// # 入力フォーマット
///
/// ```text
/// ty_fn! {
///     pub struct MyFunc<Arg1, Arg2> where [/* optional bounds */] {
///         type Output = /* 計算結果型 */;
///     }
/// }
/// ```
///
/// # 生成結果
///
/// - 構造体 `MyFunc<Arg1, Arg2>` の定義
/// - `MyFunc` に対する `Eval` 実装
/// - 必要な制約のカプセル化
///
/// # 例
///
/// ```ignore
/// ty_fn! {
///     pub struct Double<N: Nat> {
///         type Output = Mul<N, Succ<Succ<Zero>>>;
///     }
/// }
/// // Evaluate<Double<Succ<Zero>>> = Succ<Succ<Zero>> (2)
/// ```
#[proc_macro]
pub fn ty_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TyFnInput);
    TokenStream::from(input.expand_ty_fn())
}

/// 型式と Eval 実装統合マクロ。
///
/// ty_fn と同様だが、関数定義ではなく型式の評価実装を優先。
/// 計算結果をより前面に出したい場合に使用。
///
/// # 入力フォーマット
///
/// ```text
/// ty_expr! {
///     pub struct MyExpr<T> where [/* bounds */] {
///         type Output = /* 結果型 */;
///     }
/// }
/// ```
#[proc_macro]
pub fn ty_expr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TyFnInput);
    TokenStream::from(input.expand_eval())
}

/// スタックマシンプログラムコンパイルマクロ。
///
/// OpPush, OpAdd, OpIf, OpWhile 等のオペコードを用いた
/// 型レベルプログラムを記述し、コンパイル時に実行する。
///
/// # 入力フォーマット
///
/// ```text
/// program! {
///     OpPush(value),
///     OpAdd,
///     OpPop,
///     // ...
/// }
/// ```
///
/// # 結果
///
/// stack machine の実行結果を型式として返す。
#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProgramInput);
    let (expanded, _) = program::compile_block(&input.instrs, &[]);
    TokenStream::from(expanded)
}

/// WebAssembly テキスト形式→型式変換マクロ。
///
/// WASM 命令（WAT形式）を型レベル表現へ自動変換。
/// WASM の構文解析、妥当性検証、型レベルIRへのコンパイルを行う。
///
/// # 入力フォーマット
///
/// ```text
/// twat! {
///     (module
///         (func $myfunc i32.const 42 end)
///     )
/// }
/// ```
///
/// # 結果
///
/// 型レベルのWASM IR 型式を生成。
#[proc_macro]
pub fn twat(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WasmWatInput);
    match twat::expand(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => err.into_compile_error().into(),
    }
}

/// WASM テキスト形式変換マクロ（twat! のエイリアス）。
///
/// 非推奨。twat! を使用してください。
#[proc_macro]
pub fn wasm_wat(input: TokenStream) -> TokenStream {
    twat(input)
}
