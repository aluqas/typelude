use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote};
use syn::{
    Ident, LitInt, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

mod dsl;

use dsl::{TyDslInput, BoundDslInput};

// ... existing program macro code ...

enum Instruction {
    PushLiteral(LitInt),
    PushType(Type),
    SimpleOp(Ident),
    Call(Type),
    Let(Ident),
    Get(Ident),
    Set(Ident),
    Load(Ident),
    Store(Ident),
    If(Vec<Instruction>, Vec<Instruction>),
    While(Vec<Instruction>, Vec<Instruction>),
}

impl Parse for Instruction {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        if content.peek(Token![if]) {
            content.parse::<Token![if]>()?;
            let then_content;
            parenthesized!(then_content in content);
            let then_block = parse_block_content(&then_content)?;
            let else_content;
            parenthesized!(else_content in content);
            let else_block = parse_block_content(&else_content)?;
            return Ok(Instruction::If(then_block, else_block));
        }

        if content.peek(Token![while]) {
            content.parse::<Token![while]>()?;
            let cond_content;
            parenthesized!(cond_content in content);
            let cond_block = parse_block_content(&cond_content)?;
            let body_content;
            parenthesized!(body_content in content);
            let body_block = parse_block_content(&body_content)?;
            return Ok(Instruction::While(cond_block, body_block));
        }

        if content.peek(Token![let]) {
            content.parse::<Token![let]>()?;
            let v: Ident = content.parse()?;
            return Ok(Instruction::Let(v));
        }

        let op: Ident = content.parse()?;
        let s = op.to_string();
        match s.as_str() {
            "push" => {
                if content.peek(LitInt) {
                    Ok(Instruction::PushLiteral(content.parse()?))
                } else {
                    Ok(Instruction::PushType(content.parse()?))
                }
            },
            "call" => Ok(Instruction::Call(content.parse()?)),
            "get" => Ok(Instruction::Get(content.parse()?)),
            "set" => Ok(Instruction::Set(content.parse()?)),
            "load" => Ok(Instruction::Load(content.parse()?)),
            "store" => Ok(Instruction::Store(content.parse()?)),
            _ => Ok(Instruction::SimpleOp(op)),
        }
    }
}

fn parse_block_content(input: ParseStream) -> Result<Vec<Instruction>> {
    let mut instrs = Vec::new();
    while !input.is_empty() {
        instrs.push(input.parse()?);
    }
    Ok(instrs)
}

struct ProgramInput {
    instrs: Vec<Instruction>,
}

impl Parse for ProgramInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ProgramInput {
            instrs: parse_block_content(input)?,
        })
    }
}

struct CompilerState {
    locals: Vec<String>,
}

impl CompilerState {
    fn new(parent_locals: &[String]) -> Self {
        Self {
            locals: parent_locals.to_vec(),
        }
    }

    fn push_local(&mut self, name: String) {
        self.locals.push(name);
    }

    fn get_index(&self, name: &str) -> Option<usize> {
        for (i, var) in self.locals.iter().rev().enumerate() {
            if var == name {
                return Some(i);
            }
        }
        None
    }
}

fn generate_uint(n: usize) -> TokenStream2 {
    let name = format!("U{}", n);
    let ident = Ident::new(&name, proc_macro2::Span::call_site());
    quote! { typelude::typenum::#ident }
}

fn compile_instructions(
    instrs: &[Instruction],
    state: &mut CompilerState,
    cleanup: &mut Vec<TokenStream2>,
) -> Vec<TokenStream2> {
    let mut compiled = Vec::new();

    for instr in instrs {
        match instr {
            Instruction::PushLiteral(lit) => {
                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpPush<
                        <typelude::core::typenum::Const<#lit> as typelude::core::typenum::ToUInt>::Output
                    >
                });
            },
            Instruction::PushType(ty) => {
                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpPush< #ty >
                });
            },
            Instruction::SimpleOp(ident) => {
                let name = ident.to_string();
                let op_type = match name.as_str() {
                    "dup" => quote! { typelude::vm::machine::instruction::OpDup },
                    "swap" => quote! { typelude::vm::machine::instruction::OpSwap },
                    "drop" => quote! { typelude::vm::machine::instruction::OpDrop },
                    "add" => quote! { typelude::vm::machine::instruction::OpAdd },
                    "sub" => quote! { typelude::vm::machine::instruction::OpSub },
                    "eq" => quote! { typelude::vm::machine::instruction::OpEq },
                    "neq" => quote! { typelude::vm::machine::instruction::OpNeq },
                    "lt" => quote! { typelude::vm::machine::instruction::OpLt },
                    "gt" => quote! { typelude::vm::machine::instruction::OpGt },
                    "not" => quote! { typelude::vm::machine::instruction::OpNot },
                    "and" => quote! { typelude::vm::machine::instruction::OpAnd },
                    "or" => quote! { typelude::vm::machine::instruction::OpOr },
                    "return" => quote! { typelude::vm::machine::instruction::OpReturn },
                    _ => {
                        let span = ident.span();
                        return vec![
                            quote::quote_spanned! {span=> compile_error!(concat!("Unknown instruction: ", #name)); },
                        ];
                    },
                };
                compiled.push(op_type);
            },
            Instruction::Call(target) => {
                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpCall< #target >
                });
            },
            Instruction::Let(ident) => {
                state.push_local(ident.to_string());
                compiled.push(quote! { typelude::vm::machine::instruction::OpLet });
                cleanup.push(quote! { typelude::vm::machine::instruction::OpDropLocal });
            },
            Instruction::Get(ident) => match state.get_index(&ident.to_string()) {
                Some(idx) => {
                    let uint = generate_uint(idx);
                    compiled
                        .push(quote! { typelude::vm::machine::instruction::OpGetLocal< #uint > });
                },
                None => {
                    let msg = format!("Variable not found: {}", ident);
                    compiled.push(quote! { compile_error!(#msg) });
                },
            },
            Instruction::Set(ident) => match state.get_index(&ident.to_string()) {
                Some(idx) => {
                    let uint = generate_uint(idx);
                    compiled
                        .push(quote! { typelude::vm::machine::instruction::OpSetLocal< #uint > });
                },
                None => {
                    let msg = format!("Variable not found: {}", ident);
                    compiled.push(quote! { compile_error!(#msg) });
                },
            },
            Instruction::Load(ident) => {
                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpPush<#ident>,
                    typelude::vm::machine::instruction::OpLoad
                });
            },
            Instruction::Store(ident) => {
                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpPush<#ident>,
                    typelude::vm::machine::instruction::OpSwap,
                    typelude::vm::machine::instruction::OpStore
                });
            },
            Instruction::If(then_block, else_block) => {
                let (then_prog, _) = compile_block(then_block, &state.locals);
                let (else_prog, _) = compile_block(else_block, &state.locals);

                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpIf< #then_prog, #else_prog >
                });
            },
            Instruction::While(cond_block, body_block) => {
                let (cond_prog, _) = compile_block(cond_block, &state.locals);
                let (body_prog, _) = compile_block(body_block, &state.locals);

                compiled.push(quote! {
                    typelude::vm::machine::instruction::OpWhile< #cond_prog, #body_prog >
                });
            },
        }
    }
    compiled
}

fn compile_block(
    instrs: &[Instruction],
    initial_locals: &[String],
) -> (TokenStream2, Vec<TokenStream2>) {
    let mut state = CompilerState::new(initial_locals);
    let mut cleanup = Vec::new();
    let compiled_instrs = compile_instructions(instrs, &mut state, &mut cleanup);

    let mut all_instrs = compiled_instrs;

    for c in cleanup {
        all_instrs.push(c);
    }

    let expanded = quote! {
        typelude::core::tyarray![ #(#all_instrs),* ]
    };

    (expanded, Vec::new())
}

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProgramInput);
    let (output, _) = compile_block(&input.instrs, &[]);
    TokenStream::from(output)
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
