//! **def_op! Procedural Macro**
//!
//! Defines type-level operations with automatic `Eval` bounds and `Evaluate<>`
//! wrapping.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, Result, Token, Type, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Input for the def_op! macro
///
/// Supports the AST pattern:
/// `def_op! { name: DefAdd, args: (Lhs, Rhs), ast: EAdd { ... } }`
pub struct DefOpInput {
    pub doc_attrs: Vec<syn::Attribute>,
    pub op_name: Ident,
    pub args: Vec<Ident>,
    pub body: DefOpBody,
}

pub enum DefOpBody {
    Ast {
        ast_name: Ident,
        bounds: Vec<WhereBound>,
        output_ty: Type,
    },
}

/// A where bound - can be simple or complex
pub struct WhereBound {
    pub tokens: TokenStream,
}

impl Parse for DefOpInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse doc attributes
        let doc_attrs = input.call(syn::Attribute::parse_outer)?;

        // Parse "name: DefName" (identifier is only used for macro bookkeeping)
        let _name_kw: Ident = input.parse()?;
        if _name_kw != "name" {
            return Err(syn::Error::new(_name_kw.span(), "expected 'name'"));
        }
        input.parse::<Token![:]>()?;
        let op_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // Parse "args: (A, B, ...)"
        let _args_kw: Ident = input.parse()?;
        if _args_kw != "args" {
            return Err(syn::Error::new(_args_kw.span(), "expected 'args'"));
        }
        input.parse::<Token![:]>()?;
        let args_content;
        parenthesized!(args_content in input);
        let args: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&args_content)?;
        let args: Vec<Ident> = args.into_iter().collect();
        input.parse::<Token![,]>()?;

        // Parse body: "ast: Name { ... }"
        let body_kw: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let body = if body_kw != "ast" {
            return Err(syn::Error::new(body_kw.span(), "expected 'ast'"));
        } else {
            let ast_name: Ident = input.parse()?;
            let body_content;
            braced!(body_content in input);

            // Parse "where: [ ... ]"
            body_content.parse::<Token![where]>()?;
            body_content.parse::<Token![:]>()?;
            let bounds_content;
            bracketed!(bounds_content in body_content);

            // Parse bounds as token stream chunks separated by commas at bracket level 0
            let bounds = parse_where_bounds(&bounds_content)?;

            body_content.parse::<Token![,]>()?;

            // Parse "type Output = ..."
            body_content.parse::<Token![type]>()?;
            let _output_kw: Ident = body_content.parse()?;
            if _output_kw != "Output" {
                return Err(syn::Error::new(_output_kw.span(), "expected 'Output'"));
            }
            body_content.parse::<Token![=]>()?;
            let output_ty: Type = body_content.parse()?;

            DefOpBody::Ast {
                ast_name,
                bounds,
                output_ty,
            }
        };

        Ok(DefOpInput {
            doc_attrs,
            op_name,
            args,
            body,
        })
    }
}

fn parse_where_bounds(input: ParseStream) -> Result<Vec<WhereBound>> {
    let mut bounds = Vec::new();

    while !input.is_empty() {
        // Collect tokens until comma or end
        let mut tokens = TokenStream::new();
        let mut depth = 0;

        while !input.is_empty() {
            if input.peek(Token![<]) {
                depth += 1;
                let t: Token![<] = input.parse()?;
                tokens.extend(quote!(#t));
            } else if input.peek(Token![>]) {
                depth -= 1;
                let t: Token![>] = input.parse()?;
                tokens.extend(quote!(#t));
            } else if depth == 0 && input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                break;
            } else {
                let t: proc_macro2::TokenTree = input.parse()?;
                tokens.extend(quote!(#t));
            }
        }

        if !tokens.is_empty() {
            bounds.push(WhereBound {
                tokens,
            });
        }
    }

    Ok(bounds)
}

impl DefOpInput {
    /// Generate the output token stream
    pub fn expand(&self) -> TokenStream {
        let op_name = &self.op_name;
        let args = &self.args;
        let doc_attrs = &self.doc_attrs;

        match &self.body {
            DefOpBody::Ast {
                ast_name,
                bounds,
                output_ty,
            } => {
                // Generate: Arg: Eval for all args
                let eval_bounds: Vec<_> = args
                    .iter()
                    .map(|arg| {
                        quote! { #arg: typelude_std::core::Eval }
                    })
                    .collect();

                // User-specified bounds
                let user_bounds: Vec<_> = bounds.iter().map(|b| &b.tokens).collect();

                let phantom_args = if args.len() == 1 {
                    let arg = &args[0];
                    quote! { #arg }
                } else {
                    quote! { (#(#args),*) }
                };

                quote! {
                    // 1. Define AST struct
                    #(#doc_attrs)*
                    pub struct #ast_name<#(#args),*>(::std::marker::PhantomData<#phantom_args>);

                    // 2. Implement Eval for AST
                    impl<#(#args),*> typelude_std::core::Eval for #ast_name<#(#args),*>
                    where
                        #(#eval_bounds,)*
                        #(#user_bounds,)*
                    {
                        type Output = #output_ty;
                    }

                    // 3. Define Op struct
                    #(#doc_attrs)*
                    #[derive(Clone, Copy)]
                    pub struct #op_name;

                    // 4. Implement Apply for Op struct (Tuple args -> AST)
                    impl<#(#args),*> typelude_std::core::Apply<(#(#args),*)> for #op_name {
                        type Output = #ast_name<#(#args),*>;
                    }
                }
            },
        }
    }
}
