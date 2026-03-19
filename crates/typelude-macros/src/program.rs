use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, LitInt, Token, braced, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
};

pub enum Instruction {
    PushLiteral(LitInt),
    PushType(syn::Type),
    SimpleOp(Ident),
    Call(syn::Type),
    Let(Ident),
    Get(Ident),
    Set(Ident),
    Load(Ident),
    Store(Ident),
    If(Vec<Instruction>, Vec<Instruction>),
    While(Vec<Instruction>, Vec<Instruction>),
}

pub struct ProgramInput {
    pub instrs: Vec<Instruction>,
}

impl Parse for ProgramInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut instrs = Vec::new();
        while !input.is_empty() {
            instrs.push(input.parse()?);
        }
        Ok(Self { instrs })
    }
}

impl Parse for Instruction {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let instr = if content.peek(Token![let]) {
            content.parse::<Token![let]>()?;
            Instruction::Let(content.parse()?)
        } else if content.peek(Token![if]) {
            content.parse::<Token![if]>()?;
            let then_block;
            braced!(then_block in content);
            let then_instrs = parse_block(&then_block)?;
            content.parse::<Token![else]>()?;
            let else_block;
            braced!(else_block in content);
            let else_instrs = parse_block(&else_block)?;
            Instruction::If(then_instrs, else_instrs)
        } else if content.peek(Token![while]) {
            content.parse::<Token![while]>()?;
            let cond_block;
            braced!(cond_block in content);
            let cond_instrs = parse_block(&cond_block)?;
            let body_block;
            braced!(body_block in content);
            let body_instrs = parse_block(&body_block)?;
            Instruction::While(cond_instrs, body_instrs)
        } else {
            let op: Ident = content.parse()?;
            let name = op.to_string();
            match name.as_str() {
            "push" => {
                if content.peek(LitInt) {
                    Instruction::PushLiteral(content.parse()?)
                } else {
                    Instruction::PushType(content.parse()?)
                }
            },
            "dup" | "swap" | "drop" | "add" | "sub" | "eq" | "neq" | "lt" | "gt" | "not"
            | "and" | "or" | "return" => Instruction::SimpleOp(op),
            "call" => Instruction::Call(content.parse()?),
            "let" => Instruction::Let(content.parse()?),
            "get" => Instruction::Get(content.parse()?),
            "set" => Instruction::Set(content.parse()?),
            "load" => Instruction::Load(content.parse()?),
            "store" => Instruction::Store(content.parse()?),
            _ => {
                return Err(syn::Error::new_spanned(
                    op,
                    format!("Unknown instruction: {name}"),
                ));
            },
            }
        };

        Ok(instr)
    }
}

fn parse_block(input: &ParseBuffer<'_>) -> syn::Result<Vec<Instruction>> {
    let mut instrs = Vec::new();
    while !input.is_empty() {
        instrs.push(input.parse()?);
    }
    Ok(instrs)
}

struct CompilerState {
    locals: Vec<String>,
}

impl CompilerState {
    fn new(initial_locals: &[String]) -> Self {
        Self {
            locals: initial_locals.to_vec(),
        }
    }

    fn push_local(&mut self, name: String) {
        self.locals.insert(0, name);
    }

    fn get_index(&self, name: &str) -> Option<usize> {
        for (i, var) in self.locals.iter().enumerate() {
            if var == name {
                return Some(i);
            }
        }
        None
    }
}

fn generate_uint(n: usize) -> TokenStream {
    let name = format!("U{}", n);
    let ident = Ident::new(&name, proc_macro2::Span::call_site());
    quote! { typelude::typenum::#ident }
}

fn compile_instructions(
    instrs: &[Instruction],
    state: &mut CompilerState,
    cleanup: &mut Vec<TokenStream>,
) -> Vec<TokenStream> {
    let mut compiled = Vec::new();

    for instr in instrs {
        match instr {
            Instruction::PushLiteral(lit) => {
                compiled.push(quote! {
                    typelude::vm::opcode::OpPush<
                        typelude::core::ELit<
                            <typelude::core::typenum::Const<#lit> as typelude::core::typenum::ToUInt>::Output
                        >
                    >
                });
            },
            Instruction::PushType(ty) => {
                compiled.push(quote! {
                    typelude::vm::opcode::OpPush< typelude::core::ELit<#ty> >
                });
            },
            Instruction::SimpleOp(ident) => {
                let name = ident.to_string();
                let op_type = match name.as_str() {
                    "dup" => quote! { typelude::vm::opcode::OpDup },
                    "swap" => quote! { typelude::vm::opcode::OpSwap },
                    "drop" => quote! { typelude::vm::opcode::OpDrop },
                    "add" => quote! { typelude::vm::opcode::OpAdd },
                    "sub" => quote! { typelude::vm::opcode::OpSub },
                    "eq" => quote! { typelude::vm::opcode::OpEq },
                    "neq" => quote! { typelude::vm::opcode::OpNeq },
                    "lt" => quote! { typelude::vm::opcode::OpLt },
                    "gt" => quote! { typelude::vm::opcode::OpGt },
                    "not" => quote! { typelude::vm::opcode::OpNot },
                    "and" => quote! { typelude::vm::opcode::OpAnd },
                    "or" => quote! { typelude::vm::opcode::OpOr },
                    "return" => quote! { typelude::vm::opcode::OpReturn },
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
                    typelude::vm::opcode::OpCall< #target >
                });
            },
            Instruction::Let(ident) => {
                state.push_local(ident.to_string());
                compiled.push(quote! { typelude::vm::opcode::OpLet });
                cleanup.push(quote! { typelude::vm::opcode::OpDropLocal });
            },
            Instruction::Get(ident) => match state.get_index(&ident.to_string()) {
                Some(idx) => {
                    let uint = generate_uint(idx);
                    compiled.push(
                        quote! { typelude::vm::opcode::OpGetLocal< typelude::core::ELit<#uint> > },
                    );
                },
                None => {
                    let msg = format!("Variable not found: {}", ident);
                    compiled.push(quote! { compile_error!(#msg) });
                },
            },
            Instruction::Set(ident) => match state.get_index(&ident.to_string()) {
                Some(idx) => {
                    let uint = generate_uint(idx);
                    compiled.push(
                        quote! { typelude::vm::opcode::OpSetLocal< typelude::core::ELit<#uint> > },
                    );
                },
                None => {
                    let msg = format!("Variable not found: {}", ident);
                    compiled.push(quote! { compile_error!(#msg) });
                },
            },
            Instruction::Load(ident) => {
                compiled.push(quote! {
                    typelude::vm::opcode::OpPush< typelude::core::ELit<#ident> >,
                    typelude::vm::opcode::OpLoad
                });
            },
            Instruction::Store(ident) => {
                compiled.push(quote! {
                    typelude::vm::opcode::OpPush< typelude::core::ELit<#ident> >,
                    typelude::vm::opcode::OpSwap,
                    typelude::vm::opcode::OpStore
                });
            },
            Instruction::If(then_block, else_block) => {
                let (then_prog, _) = compile_block(then_block, &state.locals);
                let (else_prog, _) = compile_block(else_block, &state.locals);

                compiled.push(quote! {
                    typelude::vm::opcode::OpIf< #then_prog, #else_prog >
                });
            },
            Instruction::While(cond_block, body_block) => {
                let (cond_prog, _) = compile_block(cond_block, &state.locals);
                let (body_prog, _) = compile_block(body_block, &state.locals);

                compiled.push(quote! {
                    typelude::vm::opcode::OpWhile< #cond_prog, #body_prog >
                });
            },
        }
    }
    compiled
}

pub fn compile_block(
    instrs: &[Instruction],
    initial_locals: &[String],
) -> (TokenStream, Vec<TokenStream>) {
    let mut state = CompilerState::new(initial_locals);
    let mut cleanup = Vec::new();
    let compiled_instrs = compile_instructions(instrs, &mut state, &mut cleanup);

    let mut all_instrs = compiled_instrs;

    for c in cleanup {
        all_instrs.push(c);
    }

    let expanded = quote! {
        typelude::tyarray![ #(#all_instrs),* ]
    };

    (expanded, Vec::new())
}
