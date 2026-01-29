use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, GenericParam, Generics, Ident, Result, Token, Visibility,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};
use crate::dsl::{DslBound, DslType};

pub struct TyFnInput {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub generics: Generics,
    pub where_clause: Option<Punctuated<DslBound, Token![,]>>,
    pub brace_token: token::Brace,
    pub output_type: DslType,
}

impl Parse for TyFnInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let struct_token = input.parse()?;
        let ident = input.parse()?;

        let generics: Generics = input.parse()?;

        // Custom parsing for where clause
        let where_clause = if input.peek(Token![where]) {
            input.parse::<Token![where]>()?;
            let mut bounds = Punctuated::new();

            while !input.peek(token::Brace) && !input.is_empty() {
                // Collect tokens for ONE bound until comma or brace
                let mut tokens = TokenStream::new();
                let mut depth = 0;

                while !input.is_empty() {
                    if input.peek(token::Brace) && depth == 0 {
                        break;
                    }
                    if input.peek(Token![,]) && depth == 0 {
                        break;
                    }

                    // Removed incorrect peek-based depth modification

                    let tt: proc_macro2::TokenTree = input.parse()?;
                    match &tt {
                        proc_macro2::TokenTree::Punct(p) if p.as_char() == '<' => depth += 1,
                        proc_macro2::TokenTree::Punct(p) if p.as_char() == '>' => depth -= 1,
                        _ => {}
                    }
                    tokens.extend(std::iter::once(tt));
                }

                if !tokens.is_empty() {
                    let bound: DslBound = syn::parse2(tokens)?;
                    bounds.push_value(bound);
                }

                if input.peek(Token![,]) {
                    bounds.push_punct(input.parse()?);
                } else {
                    if !input.peek(token::Brace) && !input.is_empty() {
                         return Err(syn::Error::new(input.span(), "Unexpected end of bound parsing"));
                    }
                }
            }
            Some(bounds)
        } else {
            None
        };

        let content;
        let brace_token = syn::braced!(content in input);

        // Parse "type Output = ...;"
        content.parse::<Token![type]>()?;
        let output_ident: Ident = content.parse()?;
        if output_ident != "Output" {
            return Err(syn::Error::new(output_ident.span(), "expected 'Output'"));
        }
        content.parse::<Token![=]>()?;
        let output_type: DslType = content.parse()?;
        content.parse::<Token![;]>()?; // Consume semicolon

        Ok(TyFnInput {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            where_clause,
            brace_token,
            output_type,
        })
    }
}

impl TyFnInput {
    pub fn expand(&self) -> TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;
        let generics = &self.generics;
        let (impl_generics, type_generics, _) = generics.split_for_impl();

        // Construct PhantomData content
        let phantom_types: Vec<TokenStream> = generics.params.iter().filter_map(|p| {
            match p {
                GenericParam::Type(t) => {
                    let id = &t.ident;
                    Some(quote! { #id })
                },
                GenericParam::Lifetime(l) => {
                    let id = &l.lifetime;
                    Some(quote! { &#id () })
                },
                GenericParam::Const(_) => None,
            }
        }).collect();

        let phantom_type = if phantom_types.is_empty() {
             quote! { ::std::marker::PhantomData<()> }
        } else {
            quote! { ::std::marker::PhantomData<(#(#phantom_types),*)> }
        };

        // Construct where bounds
        let dsl_where = if let Some(bounds) = &self.where_clause {
            quote! { where #bounds }
        } else {
            quote! {}
        };

        let output_type = &self.output_type;

        // Struct definition
        let struct_def = quote! {
            #(#attrs)*
            #vis struct #ident #generics {
                pub _marker: #phantom_type
            }
        };

        // Impl Eval
        let impl_block = quote! {
            impl #impl_generics typelude_std::core::Eval for #ident #type_generics
            #dsl_where
            {
                type Output = #output_type;
            }
        };

        quote! {
            #struct_def
            #impl_block
        }
    }
}
