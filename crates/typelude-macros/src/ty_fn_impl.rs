use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Error, GenericParam, Generics, Ident, Result, Token, Visibility,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

use crate::dsl::{DslBound, DslType};

pub struct TyFnInput {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub _struct_token: Token![struct],
    pub ident: Ident,
    pub generics: Generics,
    pub where_clause: Option<Punctuated<DslBound, Token![,]>>,
    pub output_type: DslType,
    pub let_bindings: Vec<(Ident, DslType)>,
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
            if input.peek(token::Bracket) {
                let content;
                syn::bracketed!(content in input);
                let bounds: Punctuated<DslBound, Token![,]> =
                    Punctuated::parse_terminated(&content)?;
                Some(bounds)
            } else {
                let mut bounds = Punctuated::new();

                while !input.peek(token::Brace) && !input.peek(Token![=>]) && !input.is_empty() {
                    // Collect tokens for ONE bound until comma or brace/arrow
                    let mut tokens = TokenStream::new();
                    let mut depth = 0;

                    while !input.is_empty() {
                        if (input.peek(token::Brace) || input.peek(Token![=>])) && depth == 0 {
                            break;
                        }
                        if input.peek(Token![,]) && depth == 0 {
                            break;
                        }

                        let tt: proc_macro2::TokenTree = input.parse()?;
                        match &tt {
                            proc_macro2::TokenTree::Punct(p) if p.as_char() == '<' => depth += 1,
                            proc_macro2::TokenTree::Punct(p) if p.as_char() == '>' => depth -= 1,
                            _ => {},
                        }
                        tokens.extend(std::iter::once(tt));
                    }

                    if !tokens.is_empty() {
                        let bound: DslBound = syn::parse2(tokens)?;
                        bounds.push_value(bound);
                    }

                    if input.peek(Token![,]) {
                        bounds.push_punct(input.parse()?);
                    } else if !input.peek(token::Brace)
                        && !input.peek(Token![=>])
                        && !input.is_empty()
                    {
                        return Err(syn::Error::new(
                            input.span(),
                            "Unexpected end of bound parsing",
                        ));
                    }
                }
                Some(bounds)
            }
        } else {
            None
        };

        let mut let_bindings: Vec<(Ident, DslType)> = Vec::new();
        let output_type: DslType;

        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            output_type = input.parse()?;
        } else {
            let content;
            syn::braced!(content in input);
            let mut found_output = None;

            while !content.is_empty() {
                if content.peek(Token![let]) {
                    content.parse::<Token![let]>()?;
                    let name: Ident = content.parse()?;
                    content.parse::<Token![=]>()?;
                    let ty: DslType = content.parse()?;
                    content.parse::<Token![;]>()?;
                    let_bindings.push((name, ty));
                } else if content.peek(Token![type]) {
                    content.parse::<Token![type]>()?;
                    let output_ident: Ident = content.parse()?;
                    if output_ident != "Output" {
                        return Err(syn::Error::new(output_ident.span(), "expected 'Output'"));
                    }
                    if found_output.is_some() {
                        return Err(syn::Error::new(
                            output_ident.span(),
                            "duplicate 'Output' definition",
                        ));
                    }
                    content.parse::<Token![=]>()?;
                    let ty: DslType = content.parse()?;
                    content.parse::<Token![;]>()?;
                    found_output = Some(ty);
                } else {
                    return Err(syn::Error::new(
                        content.span(),
                        "expected 'let' or 'type Output = ...;'",
                    ));
                }
            }

            output_type = found_output
                .ok_or_else(|| syn::Error::new(input.span(), "missing 'type Output = ...;'"))?;
        }

        Ok(TyFnInput {
            attrs,
            vis,
            _struct_token: struct_token,
            ident,
            generics,
            where_clause,
            output_type,
            let_bindings,
        })
    }
}

fn replace_lets_in_path(
    path: &crate::dsl::DslPath,
    let_trait: &Ident,
    let_idents: &HashSet<Ident>,
) -> crate::dsl::DslPath {
    use crate::dsl::{DslGenericArguments, DslPath, DslPathSegment};
    let segments = path
        .segments
        .iter()
        .map(|seg| {
            let args = match &seg.args {
                DslGenericArguments::None => DslGenericArguments::None,
                DslGenericArguments::AngleBracketed(args) => {
                    let replaced = args
                        .iter()
                        .map(|t| replace_lets_in_type(t, let_trait, let_idents))
                        .collect();
                    DslGenericArguments::AngleBracketed(replaced)
                },
            };
            DslPathSegment {
                ident: seg.ident.clone(),
                args,
            }
        })
        .collect();

    DslPath {
        leading_colon: path.leading_colon.clone(),
        segments,
    }
}

fn replace_lets_in_type(ty: &DslType, let_trait: &Ident, let_idents: &HashSet<Ident>) -> DslType {
    use crate::dsl::{DslGenericArguments, DslPath, DslPathSegment};
    match ty {
        DslType::Base(t) => DslType::Base(t.clone()),
        DslType::Path(p) => {
            let is_simple = p.leading_colon.is_none()
                && p.segments.len() == 1
                && matches!(p.segments[0].args, DslGenericArguments::None)
                && let_idents.contains(&p.segments[0].ident);

            if is_simple {
                let self_path = DslPath {
                    leading_colon: None,
                    segments: {
                        let mut segs = Punctuated::new();
                        segs.push_value(DslPathSegment {
                            ident: Ident::new("Self", proc_macro2::Span::call_site()),
                            args: DslGenericArguments::None,
                        });
                        segs
                    },
                };
                let trait_path = DslPath {
                    leading_colon: None,
                    segments: {
                        let mut segs = Punctuated::new();
                        segs.push_value(DslPathSegment {
                            ident: let_trait.clone(),
                            args: DslGenericArguments::None,
                        });
                        segs
                    },
                };

                DslType::QSelf {
                    ty: Box::new(DslType::Path(self_path)),
                    trait_path: Some(trait_path),
                    ident: p.segments[0].ident.clone(),
                    args: None,
                }
            } else {
                DslType::Path(replace_lets_in_path(p, let_trait, let_idents))
            }
        },
        DslType::Evaluate(inner) => {
            DslType::Evaluate(Box::new(replace_lets_in_type(inner, let_trait, let_idents)))
        },
        DslType::BinaryOp(lhs, op, rhs) => DslType::BinaryOp(
            Box::new(replace_lets_in_type(lhs, let_trait, let_idents)),
            op.clone(),
            Box::new(replace_lets_in_type(rhs, let_trait, let_idents)),
        ),
        DslType::MethodCall {
            receiver,
            method,
            args,
        } => DslType::MethodCall {
            receiver: Box::new(replace_lets_in_type(receiver, let_trait, let_idents)),
            method: method.clone(),
            args: args.iter().map(|a| replace_lets_in_type(a, let_trait, let_idents)).collect(),
        },
        DslType::AssocType {
            receiver,
            ident,
        } => DslType::AssocType {
            receiver: Box::new(replace_lets_in_type(receiver, let_trait, let_idents)),
            ident: ident.clone(),
        },
        DslType::QSelf {
            ty,
            trait_path,
            ident,
            args,
        } => DslType::QSelf {
            ty: Box::new(replace_lets_in_type(ty, let_trait, let_idents)),
            trait_path: trait_path
                .as_ref()
                .map(|p| replace_lets_in_path(p, let_trait, let_idents)),
            ident: ident.clone(),
            args: args.as_ref().map(|a| {
                a.iter().map(|t| replace_lets_in_type(t, let_trait, let_idents)).collect()
            }),
        },
        DslType::Tuple(elems) => DslType::Tuple(
            elems.iter().map(|t| replace_lets_in_type(t, let_trait, let_idents)).collect(),
        ),
        DslType::Verbatim(t) => DslType::Verbatim(t.clone()),
    }
}

impl TyFnInput {
    pub fn expand_eval(&self) -> TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;
        let generics = &self.generics;
        let (impl_generics, type_generics, _) = generics.split_for_impl();

        // Construct PhantomData content
        let phantom_types: Vec<TokenStream> = generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(t) => {
                    let id = &t.ident;
                    Some(quote! { #id })
                },
                GenericParam::Lifetime(l) => {
                    let id = &l.lifetime;
                    Some(quote! { &#id () })
                },
                GenericParam::Const(_) => None,
            })
            .collect();

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

        let mut output_type = self.output_type.clone();
        let mut let_trait_block = quote! {};
        let mut let_impl_block = quote! {};

        if !self.let_bindings.is_empty() {
            let let_trait = format_ident!("__TyFnLet_{}", ident);
            let let_idents: HashSet<Ident> =
                self.let_bindings.iter().map(|(id, _)| id.clone()).collect();

            let assoc_decls = self.let_bindings.iter().map(|(name, _)| quote! { type #name; });

            let assoc_impls = self.let_bindings.iter().map(|(name, ty)| {
                let replaced = replace_lets_in_type(ty, &let_trait, &let_idents);
                quote! { type #name = #replaced; }
            });

            output_type = replace_lets_in_type(&output_type, &let_trait, &let_idents);

            let_trait_block = quote! {
                trait #let_trait {
                    #(#assoc_decls)*
                }
            };

            let_impl_block = quote! {
                impl #impl_generics #let_trait for #ident #type_generics
                #dsl_where
                {
                    #(#assoc_impls)*
                }
            };
        }

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
            #let_trait_block
            #let_impl_block
            #impl_block
        }
    }

    pub fn expand_ty_fn(&self) -> TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let ident = &self.ident;

        let mut captured_generics = self.generics.clone();
        let arg_ident = match captured_generics.params.pop() {
            Some(GenericParam::Type(arg)) => arg.ident,
            Some(other) => {
                return Error::new_spanned(
                    other,
                    "`ty_fn!` requires the last generic parameter to be the TyFn argument type",
                )
                .to_compile_error();
            },
            None => {
                return Error::new_spanned(
                    ident,
                    "`ty_fn!` requires at least one type parameter for the TyFn argument",
                )
                .to_compile_error();
            },
        };

        let (impl_generics, _, _) = self.generics.split_for_impl();
        let (_, captured_type_generics, _) = captured_generics.split_for_impl();

        let phantom_types: Vec<TokenStream> = captured_generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(t) => {
                    let id = &t.ident;
                    Some(quote! { #id })
                },
                GenericParam::Lifetime(l) => {
                    let id = &l.lifetime;
                    Some(quote! { &#id () })
                },
                GenericParam::Const(_) => None,
            })
            .collect();

        let phantom_type = if phantom_types.is_empty() {
            quote! { ::std::marker::PhantomData<()> }
        } else {
            quote! { ::std::marker::PhantomData<(#(#phantom_types),*)> }
        };

        let dsl_where = if let Some(bounds) = &self.where_clause {
            quote! { where #bounds }
        } else {
            quote! {}
        };

        let mut output_type = self.output_type.clone();
        let mut let_trait_block = quote! {};
        let mut let_impl_block = quote! {};

        if !self.let_bindings.is_empty() {
            let let_trait = format_ident!("__TyFnLet_{}", ident);
            let let_idents: HashSet<Ident> =
                self.let_bindings.iter().map(|(id, _)| id.clone()).collect();

            let assoc_decls = self.let_bindings.iter().map(|(name, _)| quote! { type #name; });
            let assoc_impls = self.let_bindings.iter().map(|(name, ty)| {
                let replaced = replace_lets_in_type(ty, &let_trait, &let_idents);
                quote! { type #name = #replaced; }
            });

            output_type = replace_lets_in_type(&output_type, &let_trait, &let_idents);

            let_trait_block = quote! {
                trait #let_trait {
                    #(#assoc_decls)*
                }
            };

            let_impl_block = quote! {
                impl #impl_generics #let_trait for #ident #captured_type_generics
                #dsl_where
                {
                    #(#assoc_impls)*
                }
            };
        }

        let struct_def = quote! {
            #(#attrs)*
            #vis struct #ident #captured_generics {
                pub _marker: #phantom_type
            }
        };

        let impl_block = quote! {
            impl #impl_generics typelude_std::core::TyFn<#arg_ident> for #ident #captured_type_generics
            #dsl_where
            {
                type Output = #output_type;
            }
        };

        quote! {
            #struct_def
            #let_trait_block
            #let_impl_block
            #impl_block
        }
    }
}
