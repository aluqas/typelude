use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Result, Token, Ident, Type, Path, token,
    punctuated::Punctuated,
    BinOp,
};

/// Represents a type expression in the DSL.
///
/// Supported syntax:
/// - `~T` -> Evaluate<T>
/// - `T` -> T (Base type)
/// - `A + B` -> <A as TypeAdd<B>>::Output
/// - `A.Func(B)` -> <A as Func<B>>::Output
/// - `A::Assoc` -> <A>::Assoc
pub enum DslType {
    Base(Type),
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
        let inner = content.parse()?;
        parse_postfix(input, inner)
    } else {
        // Parse a base type (path, identifier, etc.)
        // We speculatively parse a Type. If it fails, we error.
        // Note: `Type` parsing might be greedy, so we have to be careful about trailing dots.
        // However, `Type` usually doesn't consume `.` unless it's part of a path.
        // But `Type` *does* consume `<...>` generics.

        // Strategy: Parse a `TypePath` or similar standard Rust type.
        // Since `Type` includes `TypePath`, let's try that.
        // We need to stop before we hit generic operators if they are ambiguous, but here
        // operators like `+` are handled by `parse_binary`.
        // The issue is that `Type` might consume `+` in trait bounds, but we are in type context.
        // `Type` includes `TypeTraitObject` which has `+`.
        // We want to treat `+` as our DSL operator.
        // So we should parse an "Atomic" type, or at least `TypePath`.

        // For simplicity, let's try parsing a `TypePath` first. If user wants other types, wrap in parens?
        // Actually, `syn::Type` is fine if we check lookahead.
        // But `syn::Type` parses `A + B`. We don't want that if we want to handle `+` ourselves.
        // We'll use `TypePath` as the primary atom.

        let ty: DslType = if input.peek(Ident) || input.peek(Token![::]) || input.peek(Token![<]) {
             // Limit to TypePath-like things to allow our operators to work?
             // Or just use `Type::parse` but forbid top-level Bounds?
             // Let's rely on `Type` parsing but we might need to be careful.
             // `Type::parse` handles `+` for trait objects.
             // We want `A + B` to mean `TypeAdd`.

             // Workaround: Parse `TypePath` specifically, or `Type` but if it turns out to be a TraitObject, we might have an issue.
             // Let's restrict atoms to `TypePath`, `TypeArray`, `TypeTuple`, `TypeParen`.
             // If we use `input.parse::<TypePath>()`, it stops before `+`.
             // Note: input.parse() infers TypePath if we use syn::TypePath.
             // But map expects DslType::Base(Type).
             // So we parse TypePath, convert to Type.
             let p: syn::TypePath = input.parse()?;
             DslType::Base(Type::Path(p))
        } else {
            return Err(input.error("Expected type"));
        };

        parse_postfix(input, ty)
    }
}

fn parse_postfix(input: ParseStream, mut expr: DslType) -> Result<DslType> {
    loop {
        if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let method: Ident = input.parse()?;

            let args = if input.peek(token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let args_punctuated: Punctuated<DslType, Token![,]> = Punctuated::parse_terminated(&content)?;
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
            // We only handle simple identifier access here.
            // If `<...>` follows, it's standard Rust syntax handling in `TypePath` usually,
            // but here we are in "DSL mode".
            // `A::B` -> `<A>::B`
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
        // Check for binary operator
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
            // Not a supported binary operator
            break;
        };

        let precedence = get_precedence(&op);
        if precedence < min_precedence {
            break;
        }

        // We parsed the operator, but we need to put it back if we didn't consume it?
        // No, we parsed it inside the `if`. Wait, `input.peek` doesn't consume.
        // I need to consume it.
        // My logic above creates new tokens, but I need to actually consume from stream.
        // The `if input.peek` checks, but then I call `input.parse()?`.
        // So I am consuming.

        let rhs = parse_unary(input)?;
        // Handle right-associativity or precedence for RHS?
        let rhs = parse_binary(input, rhs, precedence + 1)?;

        lhs = DslType::BinaryOp(Box::new(lhs), op, Box::new(rhs));
    }
    Ok(lhs)
}

fn get_precedence(op: &BinOp) -> u8 {
    match op {
        BinOp::Or(_) => 10,
        BinOp::And(_) => 20,
        BinOp::BitXor(_) => 30, // Using as XOR/BitXor
        BinOp::Add(_) | BinOp::Sub(_) => 40,
        BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_) => 50,
        _ => 0,
    }
}

impl ToTokens for DslType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DslType::Base(ty) => ty.to_tokens(tokens),
            DslType::Evaluate(inner) => {
                let t = inner.as_ref();
                quote!(typelude::core::Evaluate<#t>).to_tokens(tokens);
            },
            DslType::BinaryOp(lhs, op, rhs) => {
                let trait_name = match op {
                    BinOp::Add(_) => quote!(typelude::core::std::traits::TypeAdd),
                    BinOp::Sub(_) => quote!(typelude::core::std::traits::TypeSub),
                    BinOp::Mul(_) => quote!(typelude::core::std::traits::TypeMul),
                    BinOp::Div(_) => quote!(typelude::core::std::traits::TypeDiv),
                    BinOp::Rem(_) => quote!(typelude::core::std::traits::TypeRem),
                    BinOp::And(_) => quote!(typelude::core::std::traits::TypeBool::And), // Logic traits are weird, usually `And<R>` assoc type?
                    // Wait, TypeBool trait has `type And<Rhs>`.
                    // So `<L as TypeBool>::And<R>`.
                    // But BinaryOp structure implies `Trait<Rhs>::Output`.
                    // TypeAdd is `trait TypeAdd<R> { type Output; }`.
                    // TypeBool is `trait TypeBool { type And<R>; ... }`.
                    // They are different shapes.

                    BinOp::Or(_) => quote!(typelude::core::std::traits::TypeBool::Or),
                    BinOp::BitXor(_) => quote!(typelude::core::std::traits::TypeBool::Xor), // ^ mapped to Xor

                     _ => quote!(UnknownOp),
                };

                // Check if it's a TypeBool op or Math op
                match op {
                    BinOp::And(_) | BinOp::Or(_) | BinOp::BitXor(_) => {
                         // Shape: <L as TypeBool>::Assoc<R>
                         let assoc_name = match op {
                             BinOp::And(_) => quote!(And),
                             BinOp::Or(_) => quote!(Or),
                             BinOp::BitXor(_) => quote!(Xor),
                             _ => unreachable!(),
                         };
                         quote!(< #lhs as typelude::core::std::traits::TypeBool > :: #assoc_name < #rhs >).to_tokens(tokens);
                    },
                    _ => {
                        // Shape: <L as Trait<R>>::Output
                        quote!(< #lhs as #trait_name < #rhs > > :: Output).to_tokens(tokens);
                    }
                }
            },
            DslType::MethodCall { receiver, method, args } => {
                // If args is empty: <Receiver as Method>::Output
                // If args has elements: <Receiver as Method<Args...>>::Output
                // Note: We need to handle multiple args.
                // Typically traits take one generic type, which might be a tuple.
                // `Apply<(A, B)>`

                let trait_args = if args.is_empty() {
                    quote!()
                } else if args.len() == 1 {
                    let arg = &args[0];
                    quote!(<#arg>)
                } else {
                    quote!(<(#(#args),*)>)
                };

                quote!(< #receiver as #method #trait_args > :: Output).to_tokens(tokens);
            },
            DslType::AssocType { receiver, ident } => {
                quote!(< #receiver > :: #ident).to_tokens(tokens);
            }
        }
    }
}

// =========================================================================
// Bound DSL
// =========================================================================

pub enum DslBound {
    // T: Trait
    TraitBound {
        ty: DslType,
        trait_path: Path, // We use Path to allow `std::ops::Add` etc.
    },
    // Just a type expression, implied `: Eval`
    Eval(DslType),
}

impl Parse for DslBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs = input.parse::<DslType>()?;

        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let trait_path: Path = input.parse()?;
            // We might want to parse generic args for the trait path here?
            // `Path` includes generic args (angle brackets).
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
            DslBound::TraitBound { ty, trait_path } => {
                quote!(#ty : #trait_path).to_tokens(tokens);
            },
            DslBound::Eval(ty) => {
                quote!(#ty : typelude::core::Eval).to_tokens(tokens);
            }
        }
    }
}

pub struct TyDslInput {
    pub ty: DslType,
}

impl Parse for TyDslInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TyDslInput { ty: input.parse()? })
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
