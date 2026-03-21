use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod dsl;
mod program;
mod ty_fn_impl;

use dsl::{BoundDslInput, ImplEvalInput, TyDslInput};
use program::ProgramInput;
use ty_fn_impl::TyFnInput;

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProgramInput);
    let (prog, _) = program::compile_block(&input.instrs, &[]);

    let expanded = quote! {
        <
            typelude::core::Evaluate<
                typelude::vm::vm::run::direct::ERun<
                    typelude::vm::vm::surface::aliases::ProgramVm<#prog>
                >
            >
            as typelude::vm::vm::state::GetStack
        >::Output
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn ty(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TyDslInput);
    let ty = input.ty;
    TokenStream::from(quote! { #ty })
}

#[proc_macro]
pub fn bound(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as BoundDslInput);
    let bounds = input.bounds;
    TokenStream::from(quote! { #bounds })
}

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

#[proc_macro]
pub fn ty_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TyFnInput);
    TokenStream::from(input.expand_ty_fn())
}

#[proc_macro]
pub fn ty_expr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TyFnInput);
    TokenStream::from(input.expand_eval())
}
