use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, LitStr, Token,
    parse::{Parse, ParseStream},
};
use wasmparser::{
    BlockType, ConstExpr, DataKind, ElementItems, ElementKind, ExternalKind, FuncType, Operator,
    Parser, Payload, RefType, ValType,
};

pub struct WasmWatInput {
    module: LitStr,
}

impl Parse for WasmWatInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut module = None;

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
                _ => {
                    return Err(Error::new(key.span(), "expected only the `module` field"));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            module: module
                .ok_or_else(|| Error::new(Span::call_site(), "missing `module` field"))?,
        })
    }
}

#[derive(Clone)]
struct FuncSig {
    params: Vec<Val>,
    results: Vec<Val>,
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
}

#[derive(Clone)]
enum Instr {
    Drop,
    I32Const(u32),
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
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
    CallIndirect { type_index: u32, table_index: u32 },
    Return,
    I32Load { offset: u32 },
    I32Store { offset: u32 },
    I32Load8U { offset: u32 },
    I32Store8 { offset: u32 },
    MemorySize,
    MemoryGrow,
}

struct ModuleDef {
    types: Vec<FuncSig>,
    functions: Vec<FunctionDef>,
    memory: Option<MemoryDef>,
    tables: Vec<TableDef>,
    globals: Vec<GlobalDef>,
    exports: Vec<ExportDef>,
    start: Option<u32>,
}

struct MemoryDef {
    min: u32,
    max: Option<u32>,
    data_segments: Vec<DataSegmentDef>,
}

struct TableDef {
    min: u32,
    max: Option<u32>,
    elem_segments: Vec<ElemSegmentDef>,
}

struct GlobalDef {
    mutable: bool,
    init: InitExprDef,
}

#[derive(Clone)]
enum InitExprDef {
    I32Const(u32),
}

struct DataSegmentDef {
    offset: InitExprDef,
    bytes: Vec<u32>,
}

struct ElemSegmentDef {
    table_index: u32,
    offset: InitExprDef,
    func_indices: Vec<u32>,
}

struct ExportDef {
    name: String,
    kind: ExportKind,
}

enum ExportKind {
    Func(u32),
    Memory,
    Table(u32),
}

enum Terminator {
    End,
    Else,
}

pub fn expand(input: WasmWatInput) -> syn::Result<TokenStream> {
    let wasm = wat::parse_str(input.module.value())
        .map_err(|err| Error::new(input.module.span(), format!("WAT parse error: {err}")))?;
    let module = parse_module(&wasm)?;
    let funcs = lower_func_space(&module)?;
    let memory = lower_memory_decl(module.memory.as_ref())?;
    let tables = lower_tables(&module.tables)?;
    let globals = lower_globals(&module.globals)?;
    let exports = lower_exports(&module.exports)?;
    let start = lower_start(module.start)?;

    Ok(quote!(
        ::typelude::wasm::WasmModule<
            ::typelude::wasm::TTerm,
            #funcs,
            #memory,
            #tables,
            #globals,
            #exports,
            #start,
        >
    ))
}

fn parse_module(bytes: &[u8]) -> syn::Result<ModuleDef> {
    let mut types = Vec::new();
    let mut function_type_indexes = Vec::new();
    let mut function_bodies = Vec::new();
    let mut memory = None;
    let mut saw_memory = false;
    let mut tables = Vec::new();
    let mut globals = Vec::new();
    let mut data_segments = Vec::new();
    let mut elem_segments = Vec::new();
    let mut exports = Vec::new();
    let mut start = None;

    for payload in Parser::new(0).parse_all(bytes) {
        match payload
            .map_err(|err| Error::new(Span::call_site(), format!("wasm parse error: {err}")))?
        {
            Payload::Version { encoding, .. } => {
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
                if let Some(memory_ty) = memories.first().copied() {
                    memory = Some(parse_memory_def(memory_ty)?);
                }
            },
            Payload::TableSection(reader) => {
                for table in reader {
                    tables.push(parse_table_def(table.map_err(parser_error)?)?);
                }
            },
            Payload::GlobalSection(reader) => {
                for global in reader {
                    globals.push(parse_global_def(global.map_err(parser_error)?)?);
                }
            },
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(parser_error)?;
                    let kind = match export.kind {
                        ExternalKind::Func | ExternalKind::FuncExact => {
                            ExportKind::Func(export.index)
                        },
                        ExternalKind::Memory => ExportKind::Memory,
                        ExternalKind::Table => ExportKind::Table(export.index),
                        ExternalKind::Global => {
                            return Err(Error::new(
                                Span::call_site(),
                                "unsupported export: global export is not supported",
                            ));
                        },
                        ExternalKind::Tag => {
                            return Err(Error::new(
                                Span::call_site(),
                                "unsupported export: tag export is not supported",
                            ));
                        },
                    };
                    exports.push(ExportDef {
                        name: export.name.to_owned(),
                        kind,
                    });
                }
            },
            Payload::StartSection { func, .. } => {
                start = Some(func);
            },
            Payload::ElementSection(reader) => {
                for element in reader {
                    elem_segments.push(parse_elem_segment(element.map_err(parser_error)?)?);
                }
            },
            Payload::CodeSectionStart { .. } => {},
            Payload::CodeSectionEntry(body) => {
                function_bodies.push(parse_function_body(body)?);
            },
            Payload::DataSection(reader) => {
                for data in reader {
                    data_segments.push(parse_data_segment(data.map_err(parser_error)?)?);
                }
            },
            Payload::ImportSection(_) => return unsupported_section("import"),
            Payload::DataCountSection { .. } => return unsupported_section("data_count"),
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
        });
    }

    if let Some(start_func) = start {
        let start_func = usize::try_from(start_func)
            .map_err(|_| Error::new(Span::call_site(), "start function index does not fit in usize"))?;
        let sig = functions
            .get(start_func)
            .ok_or_else(|| Error::new(Span::call_site(), "start function index out of bounds"))?
            .sig
            .clone();
        if !sig.params.is_empty() || !sig.results.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "start function must have empty params and empty results",
            ));
        }
    }

    let memory = match (memory, data_segments.is_empty()) {
        (Some(mut memory), _) => {
            memory.data_segments = data_segments;
            Some(memory)
        },
        (None, true) => None,
        (None, false) => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported section: active data requires a defined memory",
            ));
        },
    };

    if !elem_segments.is_empty() && tables.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: active elem requires a defined table",
        ));
    }
    for elem in elem_segments {
        let table = tables.get_mut(
            usize::try_from(elem.table_index)
                .map_err(|_| Error::new(Span::call_site(), "table index does not fit in usize"))?,
        )
        .ok_or_else(|| Error::new(Span::call_site(), "element table index out of bounds"))?;
        table.elem_segments.push(elem);
    }

    Ok(ModuleDef {
        types,
        functions,
        memory,
        tables,
        globals,
        exports,
        start,
    })
}

struct ParsedFunction {
    extra_locals: Vec<Val>,
    body: Vec<Instr>,
}

fn parse_function_body(body: wasmparser::FunctionBody<'_>) -> syn::Result<ParsedFunction> {
    let mut extra_locals = Vec::new();
    let locals = body.get_locals_reader().map_err(parser_error)?;
    for local in locals {
        let (count, ty) = local.map_err(parser_error)?;
        let val = lower_val_type(ty)?;
        let count = usize::try_from(count)
            .map_err(|_| Error::new(Span::call_site(), "local count does not fit in usize"))?;
        extra_locals.extend(std::iter::repeat_n(val, count));
    }

    let mut operators = body.get_operators_reader().map_err(parser_error)?;
    let (instructions, terminator) = parse_instruction_sequence(&mut operators)?;
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
    })
}

fn parse_instruction_sequence(
    operators: &mut wasmparser::OperatorsReader<'_>,
) -> syn::Result<(Vec<Instr>, Terminator)> {
    let mut instructions = Vec::new();

    loop {
        let operator = operators.read().map_err(parser_error)?;
        match operator {
            Operator::End => return Ok((instructions, Terminator::End)),
            Operator::Else => return Ok((instructions, Terminator::Else)),
            Operator::Block { blockty } => {
                ensure_empty_block_type(blockty, "block")?;
                let (body, terminator) = parse_instruction_sequence(operators)?;
                if !matches!(terminator, Terminator::End) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode else: unexpected else inside block",
                    ));
                }
                instructions.push(Instr::Block(body));
            },
            Operator::Loop { blockty } => {
                ensure_empty_block_type(blockty, "loop")?;
                let (body, terminator) = parse_instruction_sequence(operators)?;
                if !matches!(terminator, Terminator::End) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode else: unexpected else inside loop",
                    ));
                }
                instructions.push(Instr::Loop(body));
            },
            Operator::If { blockty } => {
                ensure_empty_block_type(blockty, "if")?;
                let (then_body, terminator) = parse_instruction_sequence(operators)?;
                let else_body = match terminator {
                    Terminator::End => Vec::new(),
                    Terminator::Else => {
                        let (else_body, else_terminator) = parse_instruction_sequence(operators)?;
                        if !matches!(else_terminator, Terminator::End) {
                            return Err(Error::new(
                                Span::call_site(),
                                "opcode else: malformed if/else structure",
                            ));
                        }
                        else_body
                    },
                };
                instructions.push(Instr::If(then_body, else_body));
            },
            Operator::Drop => instructions.push(Instr::Drop),
            Operator::I32Const { value } => {
                instructions.push(Instr::I32Const(parse_non_negative_i32_immediate(
                    value,
                    "opcode i32.const",
                )?));
            },
            Operator::LocalGet { local_index } => instructions.push(Instr::LocalGet(local_index)),
            Operator::LocalSet { local_index } => instructions.push(Instr::LocalSet(local_index)),
            Operator::LocalTee { local_index } => instructions.push(Instr::LocalTee(local_index)),
            Operator::GlobalGet { global_index } => {
                instructions.push(Instr::GlobalGet(global_index));
            },
            Operator::GlobalSet { global_index } => {
                instructions.push(Instr::GlobalSet(global_index));
            },
            Operator::I32Add => instructions.push(Instr::I32Add),
            Operator::I32Sub => instructions.push(Instr::I32Sub),
            Operator::I32Eqz => instructions.push(Instr::I32Eqz),
            Operator::Br { relative_depth } => instructions.push(Instr::Br(relative_depth)),
            Operator::BrIf { relative_depth } => instructions.push(Instr::BrIf(relative_depth)),
            Operator::Select => instructions.push(Instr::Select),
            Operator::TypedSelect { ty } => {
                if !matches!(lower_val_type(ty)?, Val::I32) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode typed select: only i32 typed select is supported",
                    ));
                }
                instructions.push(Instr::Select);
            },
            Operator::TypedSelectMulti { .. } => {
                return Err(Error::new(
                    Span::call_site(),
                    "opcode typed select multi: multi-value select is not supported",
                ));
            },
            Operator::Call { function_index } => instructions.push(Instr::Call(function_index)),
            Operator::CallIndirect {
                type_index,
                table_index,
            } => {
                instructions.push(Instr::CallIndirect {
                    type_index,
                    table_index,
                });
            },
            Operator::Return => instructions.push(Instr::Return),
            Operator::I32Load { memarg } => {
                instructions.push(Instr::I32Load {
                    offset: ensure_memarg(memarg, "i32.load")?,
                });
            },
            Operator::I32Store { memarg } => {
                instructions.push(Instr::I32Store {
                    offset: ensure_memarg(memarg, "i32.store")?,
                });
            },
            Operator::I32Load8U { memarg } => {
                instructions.push(Instr::I32Load8U {
                    offset: ensure_memarg(memarg, "i32.load8_u")?,
                });
            },
            Operator::I32Store8 { memarg } => {
                instructions.push(Instr::I32Store8 {
                    offset: ensure_memarg(memarg, "i32.store8")?,
                });
            },
            Operator::MemorySize { mem } => {
                if mem != 0 {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("opcode memory.size: memory index {mem} is not supported"),
                    ));
                }
                instructions.push(Instr::MemorySize);
            },
            Operator::MemoryGrow { mem } => {
                if mem != 0 {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("opcode memory.grow: memory index {mem} is not supported"),
                    ));
                }
                instructions.push(Instr::MemoryGrow);
            },
            Operator::BrTable { .. } => {
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

fn lower_func_space(module: &ModuleDef) -> syn::Result<TokenStream> {
    let types = lower_types(&module.types)?;
    let funcs = lower_functions(&module.functions)?;
    Ok(quote!(::typelude::wasm::WasmFuncSpace<#types, #funcs>))
}

fn lower_types(types: &[FuncSig]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(types.len());
    for sig in types {
        lowered.push(lower_func_type(sig)?);
    }
    Ok(lower_list(lowered))
}

fn lower_functions(functions: &[FunctionDef]) -> syn::Result<TokenStream> {
    let mut funcs = Vec::with_capacity(functions.len());
    for function in functions {
        funcs.push(lower_function(function)?);
    }
    Ok(lower_list(funcs))
}

fn lower_function(function: &FunctionDef) -> syn::Result<TokenStream> {
    let program = lower_instrs(&function.body)?;
    let func_type = lower_func_type(&function.sig)?;
    let local_inits = lower_local_inits(&function.extra_locals)?;
    Ok(quote!(::typelude::wasm::WasmFunc<#func_type, #local_inits, #program>))
}

fn lower_func_type(sig: &FuncSig) -> syn::Result<TokenStream> {
    let params = lower_val_types(&sig.params)?;
    let results = lower_val_types(&sig.results)?;
    Ok(quote!(::typelude::wasm::WasmFuncType<#params, #results>))
}

fn lower_instrs(instructions: &[Instr]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(instructions.len());
    for instr in instructions {
        lowered.push(lower_instr(instr)?);
    }
    Ok(lower_list(lowered))
}

fn lower_instr(instr: &Instr) -> syn::Result<TokenStream> {
    Ok(match instr {
        Instr::Drop => quote!(::typelude::wasm::opcode::OpDrop),
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
        Instr::GlobalGet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(::typelude::wasm::opcode::OpGlobalGet<#index>)
        },
        Instr::GlobalSet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(::typelude::wasm::opcode::OpGlobalSet<#index>)
        },
        Instr::I32Add => quote!(::typelude::wasm::opcode::OpI32Add),
        Instr::I32Sub => quote!(::typelude::wasm::opcode::OpI32Sub),
        Instr::I32Eqz => quote!(::typelude::wasm::opcode::OpI32Eqz),
        Instr::Block(body) => {
            let body = lower_instrs(body)?;
            quote!(::typelude::wasm::opcode::OpBlock<#body>)
        },
        Instr::Loop(body) => {
            let body = lower_instrs(body)?;
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
            let then_body = lower_instrs(then_body)?;
            let else_body = lower_instrs(else_body)?;
            quote!(::typelude::wasm::opcode::OpIf<#then_body, #else_body>)
        },
        Instr::Select => quote!(::typelude::wasm::opcode::OpSelect),
        Instr::Call(index) => {
            let index = uint_type(*index as usize)?;
            quote!(::typelude::wasm::opcode::OpCall<#index>)
        },
        Instr::CallIndirect {
            type_index,
            table_index,
        } => {
            let type_index = uint_type(*type_index as usize)?;
            let table_index = uint_type(*table_index as usize)?;
            quote!(::typelude::wasm::opcode::OpCallIndirect<#type_index, #table_index>)
        },
        Instr::Return => quote!(::typelude::wasm::opcode::OpReturn),
        Instr::I32Load { offset } => {
            let offset = uint_type(*offset as usize)?;
            quote!(::typelude::wasm::opcode::OpI32Load<#offset>)
        },
        Instr::I32Store { offset } => {
            let offset = uint_type(*offset as usize)?;
            quote!(::typelude::wasm::opcode::OpI32Store<#offset>)
        },
        Instr::I32Load8U { offset } => {
            let offset = uint_type(*offset as usize)?;
            quote!(::typelude::wasm::opcode::OpI32Load8U<#offset>)
        },
        Instr::I32Store8 { offset } => {
            let offset = uint_type(*offset as usize)?;
            quote!(::typelude::wasm::opcode::OpI32Store8<#offset>)
        },
        Instr::MemorySize => quote!(::typelude::wasm::opcode::OpMemorySize),
        Instr::MemoryGrow => quote!(::typelude::wasm::opcode::OpMemoryGrow),
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

fn lower_val_types(values: &[Val]) -> syn::Result<TokenStream> {
    let mut items = Vec::with_capacity(values.len());
    for value in values {
        items.push(lower_value_type(*value));
    }
    Ok(lower_list(items))
}

fn lower_value_type(val: Val) -> TokenStream {
    match val {
        Val::I32 => quote!(::typelude::wasm::WasmI32Type),
    }
}

fn lower_memory_decl(memory: Option<&MemoryDef>) -> syn::Result<TokenStream> {
    match memory {
        Some(memory) => {
            let min = uint_type(memory.min as usize)?;
            let max = lower_limit(memory.max)?;
            let data_segments = lower_data_segments(&memory.data_segments)?;
            Ok(quote!(
                ::typelude::wasm::WasmMemoryDecl<#min, #max, #data_segments>
            ))
        },
        None => Ok(quote!(
            ::typelude::wasm::WasmMemoryDecl<
                ::typelude::typenum::U0,
                ::typelude::wasm::NoLimit,
                ::typelude::wasm::TTerm,
            >
        )),
    }
}

fn lower_data_segments(data_segments: &[DataSegmentDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(data_segments.len());
    for data_segment in data_segments {
        lowered.push(lower_data_segment(data_segment)?);
    }
    Ok(lower_list(lowered))
}

fn lower_data_segment(data_segment: &DataSegmentDef) -> syn::Result<TokenStream> {
    let offset = lower_init_expr(&data_segment.offset)?;
    let bytes = lower_bytes(&data_segment.bytes)?;
    Ok(quote!(::typelude::wasm::WasmDataSegment<#offset, #bytes>))
}

fn lower_tables(tables: &[TableDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(tables.len());
    for table in tables {
        lowered.push(lower_table(table)?);
    }
    Ok(lower_list(lowered))
}

fn lower_table(table: &TableDef) -> syn::Result<TokenStream> {
    let min = uint_type(table.min as usize)?;
    let max = lower_limit(table.max)?;
    let elem_segments = lower_elem_segments(&table.elem_segments)?;
    Ok(quote!(::typelude::wasm::WasmTableDecl<#min, #max, #elem_segments>))
}

fn lower_elem_segments(elem_segments: &[ElemSegmentDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(elem_segments.len());
    for elem_segment in elem_segments {
        lowered.push(lower_elem_segment(elem_segment)?);
    }
    Ok(lower_list(lowered))
}

fn lower_elem_segment(elem_segment: &ElemSegmentDef) -> syn::Result<TokenStream> {
    let table_index = uint_type(elem_segment.table_index as usize)?;
    let offset = lower_init_expr(&elem_segment.offset)?;
    let func_indices = lower_u32_list(&elem_segment.func_indices)?;
    Ok(quote!(
        ::typelude::wasm::WasmElemSegment<#table_index, #offset, #func_indices>
    ))
}

fn lower_globals(globals: &[GlobalDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(globals.len());
    for global in globals {
        lowered.push(lower_global(global)?);
    }
    Ok(lower_list(lowered))
}

fn lower_global(global: &GlobalDef) -> syn::Result<TokenStream> {
    let mutability = if global.mutable {
        quote!(::typelude::wasm::GlobalMut)
    } else {
        quote!(::typelude::wasm::GlobalConst)
    };
    let init = lower_init_expr(&global.init)?;
    Ok(quote!(::typelude::wasm::WasmGlobalDecl<#mutability, #init>))
}

fn lower_init_expr(expr: &InitExprDef) -> syn::Result<TokenStream> {
    Ok(match expr {
        InitExprDef::I32Const(value) => {
            let value = uint_type(*value as usize)?;
            quote!(::typelude::wasm::InitI32Const<#value>)
        },
    })
}

fn lower_exports(exports: &[ExportDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(exports.len());
    for export in exports {
        lowered.push(lower_export(export)?);
    }
    Ok(lower_list(lowered))
}

fn lower_export(export: &ExportDef) -> syn::Result<TokenStream> {
    let name = LitStr::new(&export.name, Span::call_site());
    let kind = match export.kind {
        ExportKind::Func(index) => {
            let index = uint_type(index as usize)?;
            quote!(::typelude::wasm::ExportFunc<#index>)
        },
        ExportKind::Memory => quote!(::typelude::wasm::ExportMemory),
        ExportKind::Table(index) => {
            let index = uint_type(index as usize)?;
            quote!(::typelude::wasm::ExportTable<#index>)
        },
    };
    Ok(quote!(
        ::typelude::wasm::WasmExport<::typelude::wasm::tstr::TS!(#name), #kind>
    ))
}

fn lower_start(start: Option<u32>) -> syn::Result<TokenStream> {
    match start {
        Some(index) => {
            let index = uint_type(index as usize)?;
            Ok(quote!(::typelude::wasm::StartFunc<#index>))
        },
        None => Ok(quote!(::typelude::wasm::NoStart)),
    }
}

fn lower_bytes(bytes: &[u32]) -> syn::Result<TokenStream> {
    lower_u32_list(bytes)
}

fn lower_u32_list(values: &[u32]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(values.len());
    for value in values {
        lowered.push(uint_type(*value as usize)?);
    }
    Ok(lower_list(lowered))
}

fn lower_limit(limit: Option<u32>) -> syn::Result<TokenStream> {
    match limit {
        Some(limit) => uint_type(limit as usize),
        None => Ok(quote!(::typelude::wasm::NoLimit)),
    }
}

fn lower_list(items: Vec<TokenStream>) -> TokenStream {
    items.into_iter().rev().fold(
        quote!(::typelude::wasm::TTerm),
        |tail, head| quote!(::typelude::wasm::TArr<#head, #tail>),
    )
}

fn parse_memory_def(memory: wasmparser::MemoryType) -> syn::Result<MemoryDef> {
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
    Ok(MemoryDef {
        min: u32::try_from(memory.initial).map_err(|_| {
            Error::new(
                Span::call_site(),
                "unsupported section: memory minimum exceeds u32 page count",
            )
        })?,
        max: match memory.maximum {
            Some(max) => Some(u32::try_from(max).map_err(|_| {
                Error::new(
                    Span::call_site(),
                    "unsupported section: memory maximum exceeds u32 page count",
                )
            })?),
            None => None,
        },
        data_segments: Vec::new(),
    })
}

fn parse_table_def(table: wasmparser::Table<'_>) -> syn::Result<TableDef> {
    if !matches!(table.init, wasmparser::TableInit::RefNull) {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: table init expressions are not supported",
        ));
    }
    if table.ty.table64 {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: table64 is not supported",
        ));
    }
    if table.ty.shared {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: shared table is not supported",
        ));
    }
    if table.ty.element_type != RefType::FUNCREF {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: only funcref tables are supported",
        ));
    }
    Ok(TableDef {
        min: u32::try_from(table.ty.initial).map_err(|_| {
            Error::new(
                Span::call_site(),
                "unsupported section: table minimum exceeds u32 element count",
            )
        })?,
        max: match table.ty.maximum {
            Some(max) => Some(u32::try_from(max).map_err(|_| {
                Error::new(
                    Span::call_site(),
                    "unsupported section: table maximum exceeds u32 element count",
                )
            })?),
            None => None,
        },
        elem_segments: Vec::new(),
    })
}

fn parse_global_def(global: wasmparser::Global<'_>) -> syn::Result<GlobalDef> {
    if global.ty.shared {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: shared globals are not supported",
        ));
    }
    if !matches!(lower_val_type(global.ty.content_type)?, Val::I32) {
        return Err(Error::new(
            Span::call_site(),
            "unsupported type: only i32 globals are supported",
        ));
    }
    Ok(GlobalDef {
        mutable: global.ty.mutable,
        init: parse_i32_const_init_expr(&global.init_expr, "global init expr")?,
    })
}

fn parse_data_segment(data: wasmparser::Data<'_>) -> syn::Result<DataSegmentDef> {
    let (memory_index, offset_expr) = match data.kind {
        DataKind::Active {
            memory_index,
            offset_expr,
        } => (memory_index, offset_expr),
        DataKind::Passive => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported section: passive data is not supported",
            ));
        },
    };
    if memory_index != 0 {
        return Err(Error::new(
            Span::call_site(),
            format!("unsupported section: data memory index {memory_index} is not supported"),
        ));
    }
    Ok(DataSegmentDef {
        offset: parse_i32_const_init_expr(&offset_expr, "data offset expr")?,
        bytes: data.data.iter().map(|byte| u32::from(*byte)).collect(),
    })
}

fn parse_elem_segment(element: wasmparser::Element<'_>) -> syn::Result<ElemSegmentDef> {
    let (table_index, offset_expr) = match element.kind {
        ElementKind::Active {
            table_index,
            offset_expr,
        } => (table_index.unwrap_or(0), offset_expr),
        ElementKind::Passive => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported section: passive elem is not supported",
            ));
        },
        ElementKind::Declared => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported section: declarative elem is not supported",
            ));
        },
    };

    let func_indices = match element.items {
        ElementItems::Functions(reader) => reader
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(parser_error)?,
        ElementItems::Expressions(_, _) => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported section: expression elem items are not supported",
            ));
        },
    };

    Ok(ElemSegmentDef {
        table_index,
        offset: parse_i32_const_init_expr(&offset_expr, "elem offset expr")?,
        func_indices,
    })
}

fn parse_i32_const_init_expr(expr: &ConstExpr<'_>, context: &str) -> syn::Result<InitExprDef> {
    let mut operators = expr.get_operators_reader();
    let init = match operators.read().map_err(parser_error)? {
        Operator::I32Const { value } => InitExprDef::I32Const(parse_non_negative_i32_immediate(
            value,
            context,
        )?),
        other => {
            return Err(Error::new(
                Span::call_site(),
                format!("unsupported {context}: expected i32.const, found {}", opcode_name(&other)),
            ));
        },
    };
    match operators.read().map_err(parser_error)? {
        Operator::End => {},
        other => {
            return Err(Error::new(
                Span::call_site(),
                format!("unsupported {context}: expected end, found {}", opcode_name(&other)),
            ));
        },
    }
    if !operators.eof() {
        return Err(Error::new(
            Span::call_site(),
            format!("unsupported {context}: trailing operators are not supported"),
        ));
    }
    operators.finish().map_err(parser_error)?;
    Ok(init)
}

fn lower_func_sig(func: &FuncType) -> syn::Result<FuncSig> {
    let params = func
        .params()
        .iter()
        .copied()
        .map(lower_val_type)
        .collect::<syn::Result<Vec<_>>>()?;
    let results = func
        .results()
        .iter()
        .copied()
        .map(lower_val_type)
        .collect::<syn::Result<Vec<_>>>()?;
    if results.len() > 1 {
        return Err(Error::new(
            Span::call_site(),
            "unsupported type: multi-value results are not supported",
        ));
    }
    Ok(FuncSig { params, results })
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

fn ensure_memarg(memarg: wasmparser::MemArg, opcode: &str) -> syn::Result<u32> {
    if memarg.memory != 0 {
        return Err(Error::new(
            Span::call_site(),
            format!("opcode {opcode}: memory index {} is not supported", memarg.memory),
        ));
    }
    u32::try_from(memarg.offset).map_err(|_| {
        Error::new(
            Span::call_site(),
            format!("opcode {opcode}: offset {} exceeds u32", memarg.offset),
        )
    })
}

fn parse_non_negative_i32_immediate(value: i32, context: &str) -> syn::Result<u32> {
    u32::try_from(value).map_err(|_| {
        Error::new(
            Span::call_site(),
            format!("{context}: negative immediates are not supported ({value})"),
        )
    })
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
        Operator::GlobalGet { .. } => "global.get",
        Operator::GlobalSet { .. } => "global.set",
        Operator::MemoryGrow { .. } => "memory.grow",
        Operator::CallIndirect { .. } => "call_indirect",
        Operator::BrTable { .. } => "br_table",
        Operator::RefNull { .. } => "ref.null",
        Operator::Unreachable => "unreachable",
        _ => "unknown opcode",
    }
}

fn uint_type(value: usize) -> syn::Result<TokenStream> {
    let lit = syn::LitInt::new(&value.to_string(), Span::call_site());
    Ok(quote!(<::typelude::typenum::Const<#lit> as ::typelude::typenum::ToUInt>::Output))
}
