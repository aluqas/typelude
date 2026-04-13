use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;

pub fn expand(input: LitStr) -> syn::Result<TokenStream> {
    let mut output = quote!(::typelude_str::STail);
    for ch in input.value().chars().rev() {
        let ch = syn::LitChar::new(ch, Span::call_site());
        output = quote!(::typelude_str::TStr<#ch, #output>);
    }
    Ok(output)
}
