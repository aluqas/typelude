use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    BinOp, Generics, Ident, Result, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};
use syn::parse::discouraged::Speculative;

/// Represents a type expression in the DSL.
pub enum DslType {
    Base(Type),
    Path(DslPath),
    Evaluate(Box<DslType>),
    BinaryOp(Box<DslType>, BinOp, Box<DslType>),
    MethodCall {
        receiver: Box<DslType>,
        method: Ident,
        args: Vec<DslType>,
    },
    AssocType {
        receiver: Box<DslType>,
        ident: Ident,
    },
    QSelf {
        ty: Box<DslType>,
        trait_path: Option<DslPath>,
        ident: Ident,
        args: Option<Punctuated<DslType, Token![,]>>,
    },
    Tuple(Punctuated<DslType, Token![,]>),
    // Used for const expressions in generic arguments: { ... }
    Verbatim(TokenStream),
}

impl Parse for DslType {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs = parse_unary(input)?;
        parse_binary(input, lhs, 0)
    }
}

fn parse_unary(input: ParseStream) -> Result<DslType> {
    if input.peek(Token![~]) {
        input.parse::<Token![~]>()?;
        let inner = parse_unary(input)?;

        Ok(DslType::Evaluate(Box::new(inner)))
    } else if input.peek(token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let mut elems = Punctuated::new();

        while !content.is_empty() {
            elems.push_value(content.parse()?);
            if content.is_empty() {
                break;
            }
            elems.push_punct(content.parse()?);
        }

        if elems.len() == 1 && !elems.trailing_punct() {
             let first = elems.into_iter().next().unwrap();
             return parse_postfix(input, first);
        }

        parse_postfix(input, DslType::Tuple(elems))
    } else if input.peek(token::Brace) {
        // Handle { ... } for const expressions
        let content;
        syn::braced!(content in input);
        let tokens: TokenStream = content.parse()?;
        // We preserve the braces in the output
        let verbatim = quote! { { #tokens } };
        parse_postfix(input, DslType::Verbatim(verbatim))
    } else {
        // Parse a base type (path, identifier, etc.)
        let ty: DslType = if input.peek(Ident) || input.peek(Token![::])
            || input.peek(Token![crate]) || input.peek(Token![super]) || input.peek(Token![self]) {
            // Priority: Parse as DslPath structure to allow DSL inside generics
            // e.g. Vec<~T> or Result<A + B>
            let p: DslPath = input.parse()?;
            DslType::Path(p)
        } else if input.peek(Token![<]) {
            // Try to parse QSelf with DSL support first
            let fork = input.fork();
            if let Ok(qself) = parse_dsl_qself(&fork) {
                input.advance_to(&fork);
                qself
            } else {
                // Fallback to syn::TypePath
                let p: syn::TypePath = input.parse()?;
                DslType::Base(Type::Path(p))
            }
        } else {
            // Other types: tuples, references, arrays, etc.
            let t: Type = input.parse()?;
            DslType::Base(t)
        };

        parse_postfix(input, ty)
    }
}

fn parse_dsl_qself(input: ParseStream) -> Result<DslType> {
    input.parse::<Token![<]>()?;
    let ty: DslType = input.parse()?;

    let trait_path = if input.peek(Token![as]) {
        input.parse::<Token![as]>()?;
        Some(input.parse::<DslPath>()?)
    } else {
        None
    };

    input.parse::<Token![>]>()?;
    input.parse::<Token![::]>()?;
    let ident: Ident = input.parse()?;

    let args = if input.peek(Token![<]) {
        input.parse::<Token![<]>()?;
        let mut args = Punctuated::new();
        loop {
            if input.peek(Token![>]) { break; }
            args.push_value(input.parse::<DslType>()?);
            if input.peek(Token![,]) {
                args.push_punct(input.parse()?);
            } else {
                break;
            }
        }
        input.parse::<Token![>]>()?;
        Some(args)
    } else {
        None
    };

    Ok(DslType::QSelf {
        ty: Box::new(ty),
        trait_path,
        ident,
        args,
    })
}

fn parse_postfix(input: ParseStream, mut expr: DslType) -> Result<DslType> {
    loop {
        if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let method: Ident = input.parse()?;

            let args = if input.peek(token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let args_punctuated: Punctuated<DslType, Token![,]> =
                    Punctuated::parse_terminated(&content)?;
                args_punctuated.into_iter().collect()
            } else {
                Vec::new()
            };

            expr = DslType::MethodCall {
                receiver: Box::new(expr),
                method,
                args,
            };
        } else if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            let ident: Ident = input.parse()?;
            expr = DslType::AssocType {
                receiver: Box::new(expr),
                ident,
            };
        } else {
            break;
        }
    }
    Ok(expr)
}

fn parse_binary(input: ParseStream, mut lhs: DslType, min_precedence: u8) -> Result<DslType> {
    loop {
        let op = if input.peek(Token![+]) {
            BinOp::Add(input.parse()?)
        } else if input.peek(Token![-]) {
            BinOp::Sub(input.parse()?)
        } else if input.peek(Token![*]) {
            BinOp::Mul(input.parse()?)
        } else if input.peek(Token![/]) {
            BinOp::Div(input.parse()?)
        } else if input.peek(Token![%]) {
            BinOp::Rem(input.parse()?)
        } else if input.peek(Token![&&]) {
            BinOp::And(input.parse()?)
        } else if input.peek(Token![||]) {
            BinOp::Or(input.parse()?)
        } else if input.peek(Token![^]) {
            BinOp::BitXor(input.parse()?)
        } else {
            break;
        };

        let precedence = get_precedence(&op);
        if precedence < min_precedence {
            break;
        }

        let rhs = parse_unary(input)?;
        let rhs = parse_binary(input, rhs, precedence + 1)?;

        lhs = DslType::BinaryOp(Box::new(lhs), op, Box::new(rhs));
    }
    Ok(lhs)
}

fn get_precedence(op: &BinOp) -> u8 {
    match op {
        BinOp::Or(_) => 10,
        BinOp::And(_) => 20,
        BinOp::BitXor(_) => 30,
        BinOp::Add(_) | BinOp::Sub(_) => 40,
        BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_) => 50,
        _ => 0,
    }
}

impl ToTokens for DslType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DslType::Base(ty) => ty.to_tokens(tokens),
            DslType::Path(p) => p.to_tokens(tokens),
            DslType::Evaluate(inner) => {
                let t = inner.as_ref();
                quote!(typelude_std::core::Evaluate<#t>).to_tokens(tokens);
            },
            DslType::BinaryOp(lhs, op, rhs) => {
                let trait_name = match op {
                    BinOp::Add(_) => quote!(typelude_std::core::std::traits::TypeAdd),
                    BinOp::Sub(_) => quote!(typelude_std::core::std::traits::TypeSub),
                    BinOp::Mul(_) => quote!(typelude_std::core::std::traits::TypeMul),
                    BinOp::Div(_) => quote!(typelude_std::core::std::traits::TypeDiv),
                    BinOp::Rem(_) => quote!(typelude_std::core::std::traits::TypeRem),
                    BinOp::And(_) => quote!(typelude_std::core::std::traits::TypeBool::And),
                    BinOp::Or(_) => quote!(typelude_std::core::std::traits::TypeBool::Or),
                    BinOp::BitXor(_) => quote!(typelude_std::core::std::traits::TypeBool::Xor),
                    _ => quote!(UnknownOp),
                };

                match op {
                    BinOp::And(_) | BinOp::Or(_) | BinOp::BitXor(_) => {
                        let assoc_name = match op {
                            BinOp::And(_) => quote!(And),
                            BinOp::Or(_) => quote!(Or),
                            BinOp::BitXor(_) => quote!(Xor),
                            _ => unreachable!(),
                        };
                        quote!(< #lhs as typelude_std::core::std::traits::TypeBool > :: #assoc_name < #rhs >).to_tokens(tokens);
                    },
                    _ => {
                        quote!(< #lhs as #trait_name < #rhs > > :: Output).to_tokens(tokens);
                    },
                }
            },
            DslType::MethodCall {
                receiver,
                method,
                args,
            } => {
                let trait_args = if args.is_empty() {
                    quote!(<typelude_std::core::ENil>)
                } else if args.len() == 1 {
                    let arg = &args[0];
                    quote!(<#arg>)
                } else {
                    let mut stream = quote!(typelude_std::core::ENil);
                    for arg in args.iter().rev() {
                        stream = quote!(typelude_std::core::ECons<#arg, #stream>);
                    }
                    quote!(<#stream>)
                };

                quote!(< #receiver as #method #trait_args > :: Output).to_tokens(tokens);
            },
            DslType::AssocType {
                receiver,
                ident,
            } => {
                quote!(< #receiver > :: #ident).to_tokens(tokens);
            },
            DslType::QSelf { ty, trait_path, ident, args } => {
                let as_trait = if let Some(tp) = trait_path {
                     quote!(as #tp)
                } else {
                     quote!()
                };

                let gen_args = if let Some(a) = args {
                    quote!(<#a>)
                } else {
                    quote!()
                };

                quote!(< #ty #as_trait > :: #ident #gen_args).to_tokens(tokens);
            },
            DslType::Tuple(elems) => {
                quote!( ( #elems ) ).to_tokens(tokens);
            },
            DslType::Verbatim(t) => t.to_tokens(tokens),
        }
    }
}
pub struct DslPath {
    pub leading_colon: Option<Token![::]>,
    pub segments: Punctuated<DslPathSegment, Token![::]>,
}

pub struct DslPathSegment {
    pub ident: Ident,
    pub args: DslGenericArguments,
}

pub enum DslGenericArguments {
    None,
    AngleBracketed(Punctuated<DslType, Token![,]>),
    // We omit Parenthesized generic arguments (Fn(A) -> B) for simplicity in DSL for now
}

impl Parse for DslPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let leading_colon = if input.peek(Token![::]) {
            Some(input.parse()?)
        } else {
            None
        };
        let segments = Punctuated::parse_separated_nonempty(input)?;
        Ok(DslPath {
            leading_colon,
            segments,
        })
    }
}

impl Parse for DslPathSegment {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = if input.peek(Token![crate]) {
            input.parse::<Token![crate]>()?;
            Ident::new("crate", proc_macro2::Span::call_site())
        } else if input.peek(Token![super]) {
            input.parse::<Token![super]>()?;
            Ident::new("super", proc_macro2::Span::call_site())
        } else if input.peek(Token![self]) {
            input.parse::<Token![self]>()?;
            Ident::new("self", proc_macro2::Span::call_site())
        } else {
            input.parse()?
        };

        let args = if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            let mut args = Punctuated::new();
            loop {
                if input.peek(Token![>]) {
                    break;
                }
                args.push_value(input.parse()?);
                if input.peek(Token![,]) {
                    args.push_punct(input.parse()?);
                } else {
                    break;
                }
            }
            input.parse::<Token![>]>()?;
            DslGenericArguments::AngleBracketed(args)
        } else {
            DslGenericArguments::None
        };
        Ok(DslPathSegment {
            ident,
            args,
        })
    }
}

impl ToTokens for DslPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(colon) = &self.leading_colon {
            colon.to_tokens(tokens);
        }
        self.segments.to_tokens(tokens);
    }
}

impl ToTokens for DslPathSegment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        match &self.args {
            DslGenericArguments::None => {},
            DslGenericArguments::AngleBracketed(args) => {
                quote!(< #args >).to_tokens(tokens);
            },
        }
    }
}
pub enum DslBound {
    // T: Trait
    TraitBound {
        ty: DslType,
        trait_path: DslPath, // Uses DslPath to support DslType args
    },
    // Just a type expression, implied `: Eval`
    Eval(DslType),
}

impl Parse for DslBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs = input.parse::<DslType>()?;

        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let trait_path: DslPath = input.parse()?;
            Ok(DslBound::TraitBound {
                ty: lhs,
                trait_path,
            })
        } else {
            Ok(DslBound::Eval(lhs))
        }
    }
}

impl ToTokens for DslBound {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DslBound::TraitBound {
                ty,
                trait_path,
            } => {
                quote!(#ty : #trait_path).to_tokens(tokens);
            },
            DslBound::Eval(ty) => {
                quote!(#ty : typelude_std::core::Eval).to_tokens(tokens);
            },
        }
    }
}

pub struct TyDslInput {
    pub ty: DslType,
}

impl Parse for TyDslInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TyDslInput {
            ty: input.parse()?,
        })
    }
}

pub struct BoundDslInput {
    pub bounds: Punctuated<DslBound, Token![,]>,
}

impl Parse for BoundDslInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(BoundDslInput {
            bounds: Punctuated::parse_terminated(input)?,
        })
    }
}
pub struct ImplEvalInput {
    pub generics: Generics,
    pub target_type: Type,
    pub where_clause: Option<Punctuated<DslBound, Token![,]>>,
    pub output_type: DslType,
}

impl Parse for ImplEvalInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // <T> TargetType
        let generics: Generics = input.parse()?;
        let target_type: Type = input.parse()?;

        let where_clause = if input.peek(Token![where]) {
            input.parse::<Token![where]>()?;
            let mut bounds = Punctuated::new();
            loop {
                if input.peek(token::Brace) {
                    break;
                }
                bounds.push_value(input.parse()?);
                if input.peek(Token![,]) {
                    let comma = input.parse()?;
                    bounds.push_punct(comma);
                } else {
                    break;
                }
            }
            Some(bounds)
        } else {
            None
        };

        // { OutputType }
        let content;
        syn::braced!(content in input);
        let output_type: DslType = content.parse()?;

        Ok(ImplEvalInput {
            generics,
            target_type,
            where_clause,
            output_type,
        })
    }
}
