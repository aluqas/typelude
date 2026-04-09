use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, LitStr, Token, Type, bracketed,
    parse::{Parse, ParseStream},
};
use wasmparser::{BlockType, ExternalKind, FuncType, Operator, Parser, Payload, ValType};

pub struct WasmWatInput {
    module: LitStr,
    invoke: LitStr,
    args: Vec<Type>,
}

impl Parse for WasmWatInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut module = None;
        let mut invoke = None;
        let mut args = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "module" => {
                    if module.is_some() {
                        return Err(Error::new(key.span(), "`module` specified more than once"));
                    }
                    module = Some(input.parse()?);
                },
                "invoke" => {
                    if invoke.is_some() {
                        return Err(Error::new(key.span(), "`invoke` specified more than once"));
                    }
                    invoke = Some(input.parse()?);
                },
                "args" => {
                    if args.is_some() {
                        return Err(Error::new(key.span(), "`args` specified more than once"));
                    }

                    let content;
                    bracketed!(content in input);
                    let mut parsed = Vec::new();
                    while !content.is_empty() {
                        parsed.push(content.parse()?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                    args = Some(parsed);
                },
                _ => {
                    return Err(Error::new(
                        key.span(),
                        "expected one of `module`, `invoke`, or `args`",
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            module: module
                .ok_or_else(|| Error::new(Span::call_site(), "missing `module` field"))?,
            invoke: invoke
                .ok_or_else(|| Error::new(Span::call_site(), "missing `invoke` field"))?,
            args: args.unwrap_or_default(),
        })
    }
}

#[derive(Clone)]
struct FuncSig {
    params: Vec<Val>,
}

#[derive(Clone, Copy)]
enum Val {
    I32,
}

#[derive(Clone)]
struct FunctionDef {
    sig: FuncSig,
    extra_locals: Vec<Val>,
    body: Vec<Instr>,
    calls: Vec<u32>,
}

#[derive(Clone)]
enum Instr {
    I32Const(u32),
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    I32Add,
    I32Sub,
    I32Eqz,
    Block(Vec<Instr>),
    Loop(Vec<Instr>),
    Br(u32),
    BrIf(u32),
    If(Vec<Instr>, Vec<Instr>),
    Select,
    Call(u32),
    Return,
    I32Load,
    I32Store,
    I32Load8U,
    I32Store8,
    MemorySize,
}

struct ModuleDef {
    functions: Vec<FunctionDef>,
    exports: BTreeMap<String, u32>,
    memory_pages: u32,
}

enum Terminator {
    End,
    Else,
}

pub fn expand(input: WasmWatInput) -> syn::Result<TokenStream> {
    let wasm = wat::parse_str(input.module.value())
        .map_err(|err| Error::new(input.module.span(), format!("WAT parse error: {err}")))?;
    let module = parse_module(&wasm)?;

    let invoke_name = input.invoke.value();
    let function_index = module.exports.get(&invoke_name).copied().ok_or_else(|| {
        Error::new(input.invoke.span(), format!("unknown export `{invoke_name}`"))
    })?;
    let function = module
        .functions
        .get(usize::try_from(function_index).map_err(|_| {
            Error::new(input.invoke.span(), "export function index does not fit in usize")
        })?)
        .ok_or_else(|| {
            Error::new(
                input.invoke.span(),
                format!("export `{invoke_name}` does not resolve to a defined function"),
            )
        })?;

    if function.sig.params.len() != input.args.len() {
        return Err(Error::new(
            input.invoke.span(),
            format!(
                "invoke args length mismatch: export `{invoke_name}` expects {} params but got {}",
                function.sig.params.len(),
                input.args.len()
            ),
        ));
    }

    let mut cache = BTreeMap::new();
    let target = lower_function(function_index, &module, &mut cache)?;
    let memory = lower_memory(module.memory_pages);
    let stack = lower_invoke_stack(&input.args);
    let empty = quote!(::typelude::wasm::TTerm);
    let frames = quote!(
        ::typelude::wasm::TArr<
            ::typelude::wasm::ReturnFrame<#empty, #empty, #empty>,
            ::typelude::wasm::TTerm
        >
    );
    let program = lower_list(vec![quote!(::typelude::wasm::opcode::OpCall<#target>)]);
    let initial_state = quote!(
        ::typelude::wasm::WasmState<#stack, #empty, #memory, #frames, #empty, #program>
    );

    Ok(quote!(::typelude::Evaluate<::typelude::wasm::RunWasm<#initial_state>>))
}

fn parse_module(bytes: &[u8]) -> syn::Result<ModuleDef> {
    let mut types = Vec::new();
    let mut function_type_indexes = Vec::new();
    let mut function_bodies = Vec::new();
    let mut exports = BTreeMap::new();
    let mut memory_pages = 0_u32;
    let mut saw_memory = false;

    for payload in Parser::new(0).parse_all(bytes) {
        match payload
            .map_err(|err| Error::new(Span::call_site(), format!("wasm parse error: {err}")))?
        {
            Payload::Version {
                encoding,
                ..
            } => {
                if encoding != wasmparser::Encoding::Module {
                    return Err(Error::new(
                        Span::call_site(),
                        "unsupported section: component model input is not supported",
                    ));
                }
            },
            Payload::TypeSection(reader) => {
                for group in reader {
                    let group = group.map_err(parser_error)?;
                    for subtype in group.into_types() {
                        types.push(lower_func_sig(subtype.unwrap_func())?);
                    }
                }
            },
            Payload::FunctionSection(reader) => {
                for ty in reader {
                    function_type_indexes.push(ty.map_err(parser_error)?);
                }
            },
            Payload::MemorySection(reader) => {
                if saw_memory {
                    return Err(Error::new(
                        Span::call_site(),
                        "unsupported section: multiple memory sections are not supported",
                    ));
                }
                saw_memory = true;

                let memories =
                    reader.into_iter().collect::<Result<Vec<_>, _>>().map_err(parser_error)?;
                if memories.len() > 1 {
                    return Err(Error::new(
                        Span::call_site(),
                        "unsupported section: multiple memories are not supported",
                    ));
                }
                if let Some(memory) = memories.first().copied() {
                    if memory.memory64 {
                        return Err(Error::new(
                            Span::call_site(),
                            "unsupported section: memory64 is not supported",
                        ));
                    }
                    if memory.shared {
                        return Err(Error::new(
                            Span::call_site(),
                            "unsupported section: shared memory is not supported",
                        ));
                    }
                    if memory.page_size_log2.is_some() {
                        return Err(Error::new(
                            Span::call_site(),
                            "unsupported section: custom page sizes are not supported",
                        ));
                    }
                    memory_pages = u32::try_from(memory.initial).map_err(|_| {
                        Error::new(
                            Span::call_site(),
                            "unsupported section: memory minimum exceeds u32 page count",
                        )
                    })?;
                }
            },
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(parser_error)?;
                    if export.kind == ExternalKind::Func {
                        exports.insert(export.name.to_owned(), export.index);
                    }
                }
            },
            Payload::CodeSectionStart {
                ..
            } => {},
            Payload::CodeSectionEntry(body) => {
                function_bodies.push(parse_function_body(body)?);
            },
            Payload::ImportSection(_) => return unsupported_section("import"),
            Payload::GlobalSection(_) => return unsupported_section("global"),
            Payload::TableSection(_) => return unsupported_section("table"),
            Payload::ElementSection(_) => return unsupported_section("element"),
            Payload::DataSection(_) => return unsupported_section("data"),
            Payload::StartSection {
                ..
            } => return unsupported_section("start"),
            Payload::DataCountSection {
                ..
            } => return unsupported_section("data_count"),
            Payload::TagSection(_) => return unsupported_section("tag"),
            Payload::CustomSection(_) | Payload::End(_) => {},
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "unsupported section: module contains an unsupported payload",
                ));
            },
        }
    }

    if function_type_indexes.len() != function_bodies.len() {
        return Err(Error::new(
            Span::call_site(),
            "function section/code section length mismatch",
        ));
    }

    let mut functions = Vec::with_capacity(function_bodies.len());
    for (type_index, body) in function_type_indexes.into_iter().zip(function_bodies) {
        let sig = types
            .get(usize::try_from(type_index).map_err(|_| {
                Error::new(Span::call_site(), "function type index does not fit in usize")
            })?)
            .cloned()
            .ok_or_else(|| Error::new(Span::call_site(), "function type index out of bounds"))?;
        functions.push(FunctionDef {
            sig,
            extra_locals: body.extra_locals,
            body: body.body,
            calls: body.calls,
        });
    }

    ensure_non_recursive(&functions)?;

    Ok(ModuleDef {
        functions,
        exports,
        memory_pages,
    })
}

struct ParsedFunction {
    extra_locals: Vec<Val>,
    body: Vec<Instr>,
    calls: Vec<u32>,
}

fn parse_function_body(body: wasmparser::FunctionBody<'_>) -> syn::Result<ParsedFunction> {
    let mut extra_locals = Vec::new();
    let locals = body.get_locals_reader().map_err(parser_error)?;
    for local in locals {
        let (count, ty) = local.map_err(parser_error)?;
        let val = lower_val_type(ty)?;
        let count = usize::try_from(count)
            .map_err(|_| Error::new(Span::call_site(), "local count does not fit in usize"))?;
        extra_locals.extend(std::iter::repeat(val).take(count));
    }

    let mut operators = body.get_operators_reader().map_err(parser_error)?;
    let (instructions, terminator, calls) = parse_instruction_sequence(&mut operators)?;
    if !matches!(terminator, Terminator::End) {
        return Err(Error::new(
            Span::call_site(),
            "opcode else: unexpected else at function top level",
        ));
    }
    operators.finish().map_err(parser_error)?;

    Ok(ParsedFunction {
        extra_locals,
        body: instructions,
        calls,
    })
}

fn parse_instruction_sequence(
    operators: &mut wasmparser::OperatorsReader<'_>,
) -> syn::Result<(Vec<Instr>, Terminator, Vec<u32>)> {
    let mut instructions = Vec::new();
    let mut calls = Vec::new();

    loop {
        let operator = operators.read().map_err(parser_error)?;
        match operator {
            Operator::End => return Ok((instructions, Terminator::End, calls)),
            Operator::Else => return Ok((instructions, Terminator::Else, calls)),
            Operator::Block {
                blockty,
            } => {
                ensure_empty_block_type(blockty, "block")?;
                let (body, terminator, nested_calls) = parse_instruction_sequence(operators)?;
                if !matches!(terminator, Terminator::End) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode else: unexpected else inside block",
                    ));
                }
                calls.extend(nested_calls);
                instructions.push(Instr::Block(body));
            },
            Operator::Loop {
                blockty,
            } => {
                ensure_empty_block_type(blockty, "loop")?;
                let (body, terminator, nested_calls) = parse_instruction_sequence(operators)?;
                if !matches!(terminator, Terminator::End) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode else: unexpected else inside loop",
                    ));
                }
                calls.extend(nested_calls);
                instructions.push(Instr::Loop(body));
            },
            Operator::If {
                blockty,
            } => {
                ensure_empty_block_type(blockty, "if")?;
                let (then_body, terminator, then_calls) = parse_instruction_sequence(operators)?;
                calls.extend(then_calls);
                let (else_body, else_calls) = match terminator {
                    Terminator::End => (Vec::new(), Vec::new()),
                    Terminator::Else => {
                        let (else_body, else_terminator, else_calls) =
                            parse_instruction_sequence(operators)?;
                        if !matches!(else_terminator, Terminator::End) {
                            return Err(Error::new(
                                Span::call_site(),
                                "opcode else: malformed if/else structure",
                            ));
                        }
                        (else_body, else_calls)
                    },
                };
                calls.extend(else_calls);
                instructions.push(Instr::If(then_body, else_body));
            },
            Operator::I32Const {
                value,
            } => {
                if value < 0 {
                    return Err(Error::new(
                        Span::call_site(),
                        format!(
                            "opcode i32.const: negative immediates are not supported ({value})"
                        ),
                    ));
                }
                instructions.push(Instr::I32Const(value as u32));
            },
            Operator::LocalGet {
                local_index,
            } => instructions.push(Instr::LocalGet(local_index)),
            Operator::LocalSet {
                local_index,
            } => instructions.push(Instr::LocalSet(local_index)),
            Operator::LocalTee {
                local_index,
            } => instructions.push(Instr::LocalTee(local_index)),
            Operator::I32Add => instructions.push(Instr::I32Add),
            Operator::I32Sub => instructions.push(Instr::I32Sub),
            Operator::I32Eqz => instructions.push(Instr::I32Eqz),
            Operator::Br {
                relative_depth,
            } => instructions.push(Instr::Br(relative_depth)),
            Operator::BrIf {
                relative_depth,
            } => instructions.push(Instr::BrIf(relative_depth)),
            Operator::Select => instructions.push(Instr::Select),
            Operator::TypedSelect {
                ty,
            } => {
                if !matches!(lower_val_type(ty)?, Val::I32) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode typed select: only i32 typed select is supported",
                    ));
                }
                instructions.push(Instr::Select);
            },
            Operator::TypedSelectMulti {
                ..
            } => {
                return Err(Error::new(
                    Span::call_site(),
                    "opcode typed select multi: multi-value select is not supported",
                ));
            },
            Operator::Call {
                function_index,
            } => {
                calls.push(function_index);
                instructions.push(Instr::Call(function_index));
            },
            Operator::Return => instructions.push(Instr::Return),
            Operator::I32Load {
                memarg,
            } => {
                ensure_memarg(memarg, "i32.load")?;
                instructions.push(Instr::I32Load);
            },
            Operator::I32Store {
                memarg,
            } => {
                ensure_memarg(memarg, "i32.store")?;
                instructions.push(Instr::I32Store);
            },
            Operator::I32Load8U {
                memarg,
            } => {
                ensure_memarg(memarg, "i32.load8_u")?;
                instructions.push(Instr::I32Load8U);
            },
            Operator::I32Store8 {
                memarg,
            } => {
                ensure_memarg(memarg, "i32.store8")?;
                instructions.push(Instr::I32Store8);
            },
            Operator::MemorySize {
                mem,
            } => {
                if mem != 0 {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("opcode memory.size: memory index {mem} is not supported"),
                    ));
                }
                instructions.push(Instr::MemorySize);
            },
            Operator::CallIndirect {
                ..
            } => {
                return Err(Error::new(Span::call_site(), "opcode call_indirect: not supported"));
            },
            Operator::BrTable {
                ..
            } => {
                return Err(Error::new(Span::call_site(), "opcode br_table: not supported"));
            },
            other => {
                return Err(Error::new(
                    Span::call_site(),
                    format!("unsupported opcode: {}", opcode_name(&other)),
                ));
            },
        }
    }
}

fn lower_function(
    index: u32,
    module: &ModuleDef,
    cache: &mut BTreeMap<u32, TokenStream>,
) -> syn::Result<TokenStream> {
    if let Some(tokens) = cache.get(&index) {
        return Ok(tokens.clone());
    }

    let function =
        module
            .functions
            .get(usize::try_from(index).map_err(|_| {
                Error::new(Span::call_site(), "function index does not fit in usize")
            })?)
            .ok_or_else(|| {
                Error::new(Span::call_site(), format!("function index {index} out of bounds"))
            })?;

    let program = lower_instrs(&function.body, module, cache)?;
    let param_count = uint_type(function.sig.params.len())?;
    let local_inits = lower_local_inits(&function.extra_locals)?;
    let tokens = quote!(::typelude::wasm::WasmFunc<#param_count, #local_inits, #program>);
    cache.insert(index, tokens.clone());
    Ok(tokens)
}

fn lower_instrs(
    instructions: &[Instr],
    module: &ModuleDef,
    cache: &mut BTreeMap<u32, TokenStream>,
) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(instructions.len());
    for instr in instructions {
        lowered.push(lower_instr(instr, module, cache)?);
    }
    Ok(lower_list(lowered))
}

fn lower_instr(
    instr: &Instr,
    module: &ModuleDef,
    cache: &mut BTreeMap<u32, TokenStream>,
) -> syn::Result<TokenStream> {
    Ok(match instr {
        Instr::I32Const(value) => {
            let value = uint_type(*value as usize)?;
            quote!(::typelude::wasm::opcode::OpI32Const<#value>)
        },
        Instr::LocalGet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(::typelude::wasm::opcode::OpLocalGet<#index>)
        },
        Instr::LocalSet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(::typelude::wasm::opcode::OpLocalSet<#index>)
        },
        Instr::LocalTee(index) => {
            let index = uint_type(*index as usize)?;
            quote!(::typelude::wasm::opcode::OpLocalTee<#index>)
        },
        Instr::I32Add => quote!(::typelude::wasm::opcode::OpI32Add),
        Instr::I32Sub => quote!(::typelude::wasm::opcode::OpI32Sub),
        Instr::I32Eqz => quote!(::typelude::wasm::opcode::OpI32Eqz),
        Instr::Block(body) => {
            let body = lower_instrs(body, module, cache)?;
            quote!(::typelude::wasm::opcode::OpBlock<#body>)
        },
        Instr::Loop(body) => {
            let body = lower_instrs(body, module, cache)?;
            quote!(::typelude::wasm::opcode::OpLoop<#body>)
        },
        Instr::Br(depth) => {
            let depth = uint_type(*depth as usize)?;
            quote!(::typelude::wasm::opcode::OpBr<#depth>)
        },
        Instr::BrIf(depth) => {
            let depth = uint_type(*depth as usize)?;
            quote!(::typelude::wasm::opcode::OpBrIf<#depth>)
        },
        Instr::If(then_body, else_body) => {
            let then_body = lower_instrs(then_body, module, cache)?;
            let else_body = lower_instrs(else_body, module, cache)?;
            quote!(::typelude::wasm::opcode::OpIf<#then_body, #else_body>)
        },
        Instr::Select => quote!(::typelude::wasm::opcode::OpSelect),
        Instr::Call(index) => {
            let func = lower_function(*index, module, cache)?;
            quote!(::typelude::wasm::opcode::OpCall<#func>)
        },
        Instr::Return => quote!(::typelude::wasm::opcode::OpReturn),
        Instr::I32Load => quote!(::typelude::wasm::opcode::OpI32Load),
        Instr::I32Store => quote!(::typelude::wasm::opcode::OpI32Store),
        Instr::I32Load8U => quote!(::typelude::wasm::opcode::OpI32Load8U),
        Instr::I32Store8 => quote!(::typelude::wasm::opcode::OpI32Store8),
        Instr::MemorySize => quote!(::typelude::wasm::opcode::OpMemorySize),
    })
}

fn lower_local_inits(locals: &[Val]) -> syn::Result<TokenStream> {
    let mut items = Vec::with_capacity(locals.len());
    for local in locals {
        items.push(lower_zero_init(*local));
    }
    Ok(lower_list(items))
}

fn lower_zero_init(val: Val) -> TokenStream {
    match val {
        Val::I32 => quote!(
            ::typelude::wasm::WasmI32<
                <::typelude::typenum::Const<0> as ::typelude::typenum::ToUInt>::Output,
            >
        ),
    }
}

fn lower_memory(pages: u32) -> TokenStream {
    let pages = uint_type(pages as usize).expect("memory pages should fit in typenum Const");
    quote!(::typelude::wasm::WasmMemory<#pages, ::typelude::wasm::TTerm>)
}

fn lower_invoke_stack(args: &[Type]) -> TokenStream {
    let items =
        args.iter().rev().map(|arg| quote!(::typelude::wasm::WasmI32<#arg>)).collect::<Vec<_>>();
    lower_list(items)
}

fn lower_list(items: Vec<TokenStream>) -> TokenStream {
    items.into_iter().rev().fold(
        quote!(::typelude::wasm::TTerm),
        |tail, head| quote!(::typelude::wasm::TArr<#head, #tail>),
    )
}

fn lower_func_sig(func: &FuncType) -> syn::Result<FuncSig> {
    let params =
        func.params().iter().copied().map(lower_val_type).collect::<syn::Result<Vec<_>>>()?;
    let results =
        func.results().iter().copied().map(lower_val_type).collect::<syn::Result<Vec<_>>>()?;
    if results.len() > 1 {
        return Err(Error::new(
            Span::call_site(),
            "unsupported type: multi-value results are not supported",
        ));
    }
    Ok(FuncSig {
        params,
    })
}

fn lower_val_type(ty: ValType) -> syn::Result<Val> {
    match ty {
        ValType::I32 => Ok(Val::I32),
        ValType::I64 => Err(Error::new(Span::call_site(), "unsupported type: i64")),
        ValType::F32 => Err(Error::new(Span::call_site(), "unsupported type: f32")),
        ValType::F64 => Err(Error::new(Span::call_site(), "unsupported type: f64")),
        ValType::V128 => Err(Error::new(Span::call_site(), "unsupported type: v128")),
        ValType::Ref(_) => Err(Error::new(Span::call_site(), "unsupported type: reference type")),
    }
}

fn ensure_empty_block_type(block_type: BlockType, opcode: &str) -> syn::Result<()> {
    match block_type {
        BlockType::Empty => Ok(()),
        BlockType::Type(ty) => Err(Error::new(
            Span::call_site(),
            format!("opcode {opcode}: block results are not supported ({ty:?})"),
        )),
        BlockType::FuncType(index) => Err(Error::new(
            Span::call_site(),
            format!("opcode {opcode}: func-typed blocks are not supported (type {index})"),
        )),
    }
}

fn ensure_memarg(memarg: wasmparser::MemArg, opcode: &str) -> syn::Result<()> {
    if memarg.memory != 0 {
        return Err(Error::new(
            Span::call_site(),
            format!("opcode {opcode}: memory index {} is not supported", memarg.memory),
        ));
    }
    if memarg.offset != 0 {
        return Err(Error::new(
            Span::call_site(),
            format!("opcode {opcode}: offset {} is not supported", memarg.offset),
        ));
    }
    Ok(())
}

fn ensure_non_recursive(functions: &[FunctionDef]) -> syn::Result<()> {
    fn visit(
        index: usize,
        functions: &[FunctionDef],
        visiting: &mut BTreeSet<usize>,
        visited: &mut BTreeSet<usize>,
    ) -> syn::Result<()> {
        if visited.contains(&index) {
            return Ok(());
        }
        if !visiting.insert(index) {
            return Err(Error::new(
                Span::call_site(),
                format!("recursive function call cycle detected at function index {index}"),
            ));
        }

        for &callee in &functions[index].calls {
            let callee = usize::try_from(callee).map_err(|_| {
                Error::new(Span::call_site(), "callee index does not fit in usize")
            })?;
            if callee >= functions.len() {
                return Err(Error::new(
                    Span::call_site(),
                    format!("call target function index {callee} is out of bounds"),
                ));
            }
            visit(callee, functions, visiting, visited)?;
        }

        visiting.remove(&index);
        visited.insert(index);
        Ok(())
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for index in 0..functions.len() {
        visit(index, functions, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn unsupported_section<T>(name: &str) -> syn::Result<T> {
    Err(Error::new(Span::call_site(), format!("unsupported section: {name}")))
}

fn parser_error(err: wasmparser::BinaryReaderError) -> Error {
    Error::new(Span::call_site(), format!("wasm parse error: {err}"))
}

fn opcode_name(operator: &Operator<'_>) -> &'static str {
    match operator {
        Operator::Drop => "drop",
        Operator::Nop => "nop",
        Operator::MemoryGrow {
            ..
        } => "memory.grow",
        Operator::CallIndirect {
            ..
        } => "call_indirect",
        Operator::BrTable {
            ..
        } => "br_table",
        Operator::Unreachable => "unreachable",
        _ => "unknown opcode",
    }
}

fn uint_type(value: usize) -> syn::Result<TokenStream> {
    let lit = syn::LitInt::new(&value.to_string(), Span::call_site());
    Ok(quote!(<::typelude::typenum::Const<#lit> as ::typelude::typenum::ToUInt>::Output))
}
