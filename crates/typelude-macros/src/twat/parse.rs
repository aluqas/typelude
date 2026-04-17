use proc_macro2::Span;
use syn::Error;
use wasmparser::{
    BlockType, ConstExpr, DataKind, ElementItems, ElementKind, ExternalKind, FuncType, Imports,
    Operator, Parser, Payload, RefType, TypeRef, ValType,
};

use super::ir::{
    ConstExprDef, ConstInstrDef, DataSegmentDef, ElemSegmentDef, ExportDef, ExportKind, FuncSig,
    FunctionDef, GlobalDef, ImportDef, ImportKindDef, Instr, MemArgDef, MemoryDef, ModuleDef,
    TableDef, Terminator, Val,
};

struct ParsedFunction {
    local_decls: Vec<Val>,
    body: Vec<Instr>,
}

pub fn parse_module(bytes: &[u8]) -> syn::Result<ModuleDef> {
    let mut imports = Vec::new();
    let mut types = Vec::new();
    let mut function_type_indexes = Vec::new();
    let mut function_bodies = Vec::new();
    let mut memory = None;
    let mut tables = Vec::new();
    let mut globals = Vec::new();
    let mut data_segments = Vec::new();
    let mut elem_segments = Vec::new();
    let mut exports = Vec::new();
    let mut start = None;

    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(parser_error)? {
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
            Payload::ImportSection(reader) => {
                for group in reader {
                    parse_import_group(group.map_err(parser_error)?, &types, &mut imports)?;
                }
            },
            Payload::FunctionSection(reader) => {
                for ty in reader {
                    function_type_indexes.push(ty.map_err(parser_error)?);
                }
            },
            Payload::MemorySection(reader) => {
                if memory.is_some() {
                    return Err(Error::new(
                        Span::call_site(),
                        "unsupported section: multiple memory sections are not supported",
                    ));
                }
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
                        ExternalKind::Global => ExportKind::Global(export.index),
                        ExternalKind::Memory => ExportKind::Memory,
                        ExternalKind::Table => ExportKind::Table(export.index),
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
            Payload::StartSection {
                func,
                ..
            } => start = Some(func),
            Payload::ElementSection(reader) => {
                for element in reader {
                    elem_segments.push(parse_elem_segment(element.map_err(parser_error)?)?);
                }
            },
            Payload::CodeSectionStart {
                ..
            } => {},
            Payload::CodeSectionEntry(body) => function_bodies.push(parse_function_body(body)?),
            Payload::DataSection(reader) => {
                for data in reader {
                    data_segments.push(parse_data_segment(data.map_err(parser_error)?)?);
                }
            },
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
            local_decls: body.local_decls,
            body: body.body,
        });
    }

    Ok(ModuleDef {
        imports,
        types,
        functions,
        memory,
        tables,
        globals,
        data_segments,
        elem_segments,
        exports,
        start,
    })
}

fn parse_import_group(
    group: Imports<'_>,
    types: &[FuncSig],
    imports: &mut Vec<ImportDef>,
) -> syn::Result<()> {
    match group {
        Imports::Single(_, import) => {
            parse_import_item(import.module, import.name, import.ty, types, imports)
        },
        Imports::Compact1 {
            module,
            items,
        } => {
            for item in items {
                let item = item.map_err(parser_error)?;
                parse_import_item(module, item.name, item.ty, types, imports)?;
            }
            Ok(())
        },
        Imports::Compact2 {
            module,
            ty,
            names,
        } => {
            for name in names {
                parse_import_item(module, name.map_err(parser_error)?, ty, types, imports)?;
            }
            Ok(())
        },
    }
}

fn parse_import_item(
    module_name: &str,
    field_name: &str,
    ty: TypeRef,
    types: &[FuncSig],
    imports: &mut Vec<ImportDef>,
) -> syn::Result<()> {
    let kind = match ty {
        TypeRef::Func(type_index) | TypeRef::FuncExact(type_index) => {
            let sig = types
                .get(usize::try_from(type_index).map_err(|_| {
                    Error::new(
                        Span::call_site(),
                        "import function type index does not fit in usize",
                    )
                })?)
                .cloned()
                .ok_or_else(|| {
                    Error::new(Span::call_site(), "import function type index out of bounds")
                })?;
            ImportKindDef::Func(sig)
        },
        TypeRef::Global(global_ty) => parse_global_import_kind(global_ty)?,
        TypeRef::Memory(memory_ty) => {
            let memory_def = parse_memory_def(memory_ty)?;
            ImportKindDef::Memory {
                min: memory_def.min,
                max: memory_def.max,
            }
        },
        TypeRef::Table(table_ty) => {
            let table_def = parse_table_type(table_ty)?;
            ImportKindDef::Table {
                min: table_def.min,
                max: table_def.max,
            }
        },
        TypeRef::Tag(_) => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported import: tag import is not supported",
            ));
        },
    };

    imports.push(ImportDef {
        module: module_name.to_owned(),
        field: field_name.to_owned(),
        kind,
    });
    Ok(())
}

fn parse_global_import_kind(global_ty: wasmparser::GlobalType) -> syn::Result<ImportKindDef> {
    if global_ty.shared {
        return Err(Error::new(
            Span::call_site(),
            "unsupported import: shared globals are not supported",
        ));
    }
    Ok(ImportKindDef::Global {
        mutable: global_ty.mutable,
        value_type: lower_val_type(global_ty.content_type)?,
    })
}

fn parse_function_body(body: wasmparser::FunctionBody<'_>) -> syn::Result<ParsedFunction> {
    let mut local_decls = Vec::new();
    let locals = body.get_locals_reader().map_err(parser_error)?;
    for local in locals {
        let (count, ty) = local.map_err(parser_error)?;
        let val = lower_val_type(ty)?;
        let count = usize::try_from(count)
            .map_err(|_| Error::new(Span::call_site(), "local count does not fit in usize"))?;
        local_decls.extend(std::iter::repeat_n(val, count));
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
        local_decls,
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
            Operator::Block {
                blockty,
            } => {
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
            Operator::Loop {
                blockty,
            } => {
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
            Operator::If {
                blockty,
            } => {
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
            Operator::Nop => instructions.push(Instr::Nop),
            Operator::Unreachable => instructions.push(Instr::Unreachable),
            Operator::I32Const {
                value,
            } => instructions.push(Instr::I32Const(i32_to_bitpattern(value))),
            Operator::I64Const {
                value,
            } => instructions.push(Instr::I64Const(i64_to_bitpattern(value))),
            Operator::LocalGet {
                local_index,
            } => instructions.push(Instr::LocalGet(local_index)),
            Operator::LocalSet {
                local_index,
            } => instructions.push(Instr::LocalSet(local_index)),
            Operator::LocalTee {
                local_index,
            } => instructions.push(Instr::LocalTee(local_index)),
            Operator::GlobalGet {
                global_index,
            } => instructions.push(Instr::GlobalGet(global_index)),
            Operator::GlobalSet {
                global_index,
            } => instructions.push(Instr::GlobalSet(global_index)),
            Operator::I32Add => instructions.push(Instr::I32Add),
            Operator::I32And => instructions.push(Instr::I32And),
            Operator::I32Clz => instructions.push(Instr::I32Clz),
            Operator::I32Sub => instructions.push(Instr::I32Sub),
            Operator::I32Ctz => instructions.push(Instr::I32Ctz),
            Operator::I32DivS => instructions.push(Instr::I32DivS),
            Operator::I32DivU => instructions.push(Instr::I32DivU),
            Operator::I32Eq => instructions.push(Instr::I32Eq),
            Operator::I32Eqz => instructions.push(Instr::I32Eqz),
            Operator::I32Extend8S => instructions.push(Instr::I32Extend8S),
            Operator::I32Extend16S => instructions.push(Instr::I32Extend16S),
            Operator::I32GeS => instructions.push(Instr::I32GeS),
            Operator::I32GeU => instructions.push(Instr::I32GeU),
            Operator::I32GtS => instructions.push(Instr::I32GtS),
            Operator::I32GtU => instructions.push(Instr::I32GtU),
            Operator::I32LeS => instructions.push(Instr::I32LeS),
            Operator::I32LeU => instructions.push(Instr::I32LeU),
            Operator::I32LtS => instructions.push(Instr::I32LtS),
            Operator::I32LtU => instructions.push(Instr::I32LtU),
            Operator::I32Mul => instructions.push(Instr::I32Mul),
            Operator::I32Ne => instructions.push(Instr::I32Ne),
            Operator::I32Or => instructions.push(Instr::I32Or),
            Operator::I32Popcnt => instructions.push(Instr::I32Popcnt),
            Operator::I32RemS => instructions.push(Instr::I32RemS),
            Operator::I32RemU => instructions.push(Instr::I32RemU),
            Operator::I32Rotl => instructions.push(Instr::I32Rotl),
            Operator::I32Rotr => instructions.push(Instr::I32Rotr),
            Operator::I32Shl => instructions.push(Instr::I32Shl),
            Operator::I32ShrS => instructions.push(Instr::I32ShrS),
            Operator::I32ShrU => instructions.push(Instr::I32ShrU),
            Operator::I32WrapI64 => instructions.push(Instr::I32WrapI64),
            Operator::I32Xor => instructions.push(Instr::I32Xor),
            Operator::F32ReinterpretI32 => instructions.push(Instr::F32ReinterpretI32),
            Operator::F64ReinterpretI64 => instructions.push(Instr::F64ReinterpretI64),
            Operator::I64Add => instructions.push(Instr::I64Add),
            Operator::I64And => instructions.push(Instr::I64And),
            Operator::I64Clz => instructions.push(Instr::I64Clz),
            Operator::I64Sub => instructions.push(Instr::I64Sub),
            Operator::I64Ctz => instructions.push(Instr::I64Ctz),
            Operator::I64Eqz => instructions.push(Instr::I64Eqz),
            Operator::I64Eq => instructions.push(Instr::I64Eq),
            Operator::I64ExtendI32S => instructions.push(Instr::I64ExtendI32S),
            Operator::I64ExtendI32U => instructions.push(Instr::I64ExtendI32U),
            Operator::I64Ne => instructions.push(Instr::I64Ne),
            Operator::I64LtS => instructions.push(Instr::I64LtS),
            Operator::I64LtU => instructions.push(Instr::I64LtU),
            Operator::I64GtS => instructions.push(Instr::I64GtS),
            Operator::I64GtU => instructions.push(Instr::I64GtU),
            Operator::I64LeS => instructions.push(Instr::I64LeS),
            Operator::I64LeU => instructions.push(Instr::I64LeU),
            Operator::I64GeS => instructions.push(Instr::I64GeS),
            Operator::I64GeU => instructions.push(Instr::I64GeU),
            Operator::I64Or => instructions.push(Instr::I64Or),
            Operator::I64Popcnt => instructions.push(Instr::I64Popcnt),
            Operator::I64Xor => instructions.push(Instr::I64Xor),
            Operator::I64Shl => instructions.push(Instr::I64Shl),
            Operator::I64ShrS => instructions.push(Instr::I64ShrS),
            Operator::I64ShrU => instructions.push(Instr::I64ShrU),
            Operator::I64Rotl => instructions.push(Instr::I64Rotl),
            Operator::I64Rotr => instructions.push(Instr::I64Rotr),
            Operator::I64Mul => instructions.push(Instr::I64Mul),
            Operator::I64DivS => instructions.push(Instr::I64DivS),
            Operator::I64DivU => instructions.push(Instr::I64DivU),
            Operator::I64RemS => instructions.push(Instr::I64RemS),
            Operator::I64RemU => instructions.push(Instr::I64RemU),
            Operator::I64ReinterpretF64 => instructions.push(Instr::I64ReinterpretF64),
            Operator::Br {
                relative_depth,
            } => instructions.push(Instr::Br(relative_depth)),
            Operator::BrIf {
                relative_depth,
            } => instructions.push(Instr::BrIf(relative_depth)),
            Operator::BrTable {
                targets,
            } => instructions.push(Instr::BrTable {
                targets: targets.targets().collect::<Result<Vec<_>, _>>().map_err(parser_error)?,
                default: targets.default(),
            }),
            Operator::Select => instructions.push(Instr::Select),
            Operator::TypedSelect {
                ty,
            } => {
                if !matches!(lower_val_type(ty)?, Val::I32 | Val::I64) {
                    return Err(Error::new(
                        Span::call_site(),
                        "opcode typed select: only i32/i64 typed select is supported",
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
            } => instructions.push(Instr::Call(function_index)),
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
            Operator::I32Load {
                memarg,
            } => instructions.push(Instr::I32Load(parse_memarg(memarg, "i32.load")?)),
            Operator::I32Store {
                memarg,
            } => instructions.push(Instr::I32Store(parse_memarg(memarg, "i32.store")?)),
            Operator::I32Load8S {
                memarg,
            } => instructions.push(Instr::I32Load8S(parse_memarg(memarg, "i32.load8_s")?)),
            Operator::I32Load8U {
                memarg,
            } => instructions.push(Instr::I32Load8U(parse_memarg(memarg, "i32.load8_u")?)),
            Operator::I32Load16S {
                memarg,
            } => instructions.push(Instr::I32Load16S(parse_memarg(memarg, "i32.load16_s")?)),
            Operator::I32Load16U {
                memarg,
            } => instructions.push(Instr::I32Load16U(parse_memarg(memarg, "i32.load16_u")?)),
            Operator::I32Store8 {
                memarg,
            } => instructions.push(Instr::I32Store8(parse_memarg(memarg, "i32.store8")?)),
            Operator::I32Store16 {
                memarg,
            } => instructions.push(Instr::I32Store16(parse_memarg(memarg, "i32.store16")?)),
            Operator::I64Load8S {
                memarg,
            } => instructions.push(Instr::I64Load8S(parse_memarg(memarg, "i64.load8_s")?)),
            Operator::I64Load8U {
                memarg,
            } => instructions.push(Instr::I64Load8U(parse_memarg(memarg, "i64.load8_u")?)),
            Operator::I64Load16S {
                memarg,
            } => instructions.push(Instr::I64Load16S(parse_memarg(memarg, "i64.load16_s")?)),
            Operator::I64Load16U {
                memarg,
            } => instructions.push(Instr::I64Load16U(parse_memarg(memarg, "i64.load16_u")?)),
            Operator::I64Load32S {
                memarg,
            } => instructions.push(Instr::I64Load32S(parse_memarg(memarg, "i64.load32_s")?)),
            Operator::I64Load32U {
                memarg,
            } => instructions.push(Instr::I64Load32U(parse_memarg(memarg, "i64.load32_u")?)),
            Operator::I64Load {
                memarg,
            } => instructions.push(Instr::I64Load(parse_memarg(memarg, "i64.load")?)),
            Operator::I64Store8 {
                memarg,
            } => instructions.push(Instr::I64Store8(parse_memarg(memarg, "i64.store8")?)),
            Operator::I64Store16 {
                memarg,
            } => instructions.push(Instr::I64Store16(parse_memarg(memarg, "i64.store16")?)),
            Operator::I64Store32 {
                memarg,
            } => instructions.push(Instr::I64Store32(parse_memarg(memarg, "i64.store32")?)),
            Operator::I64Store {
                memarg,
            } => instructions.push(Instr::I64Store(parse_memarg(memarg, "i64.store")?)),
            Operator::MemorySize {
                mem,
            } => instructions.push(Instr::MemorySize(mem)),
            Operator::MemoryGrow {
                mem,
            } => instructions.push(Instr::MemoryGrow(mem)),
            other => {
                return Err(Error::new(
                    Span::call_site(),
                    format!("unsupported opcode: {}", opcode_name(&other)),
                ));
            },
        }
    }
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
    })
}

fn parse_table_def(table: wasmparser::Table<'_>) -> syn::Result<TableDef> {
    if !matches!(table.init, wasmparser::TableInit::RefNull) {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: table init expressions are not supported",
        ));
    }
    parse_table_type(table.ty)
}

fn parse_table_type(table_ty: wasmparser::TableType) -> syn::Result<TableDef> {
    if table_ty.table64 {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: table64 is not supported",
        ));
    }
    if table_ty.shared {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: shared table is not supported",
        ));
    }
    if table_ty.element_type != RefType::FUNCREF {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: only funcref tables are supported",
        ));
    }
    Ok(TableDef {
        min: u32::try_from(table_ty.initial).map_err(|_| {
            Error::new(
                Span::call_site(),
                "unsupported section: table minimum exceeds u32 element count",
            )
        })?,
        max: match table_ty.maximum {
            Some(max) => Some(u32::try_from(max).map_err(|_| {
                Error::new(
                    Span::call_site(),
                    "unsupported section: table maximum exceeds u32 element count",
                )
            })?),
            None => None,
        },
    })
}

fn parse_global_def(global: wasmparser::Global<'_>) -> syn::Result<GlobalDef> {
    if global.ty.shared {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: shared globals are not supported",
        ));
    }
    lower_val_type(global.ty.content_type)?;
    Ok(GlobalDef {
        mutable: global.ty.mutable,
        init: parse_const_expr(&global.init_expr, "global init expr")?,
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
        offset: parse_const_expr(&offset_expr, "data offset expr")?,
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
        ElementItems::Functions(reader) => {
            reader.into_iter().collect::<Result<Vec<_>, _>>().map_err(parser_error)?
        },
        ElementItems::Expressions(_, _) => {
            return Err(Error::new(
                Span::call_site(),
                "unsupported section: expression elem items are not supported",
            ));
        },
    };

    Ok(ElemSegmentDef {
        table_index,
        offset: parse_const_expr(&offset_expr, "elem offset expr")?,
        func_indices,
    })
}

fn parse_const_expr(expr: &ConstExpr<'_>, context: &str) -> syn::Result<ConstExprDef> {
    let mut operators = expr.get_operators_reader();
    let init = match operators.read().map_err(parser_error)? {
        Operator::I32Const {
            value,
        } => ConstInstrDef::I32Const(i32_to_bitpattern(value)),
        Operator::I64Const {
            value,
        } => ConstInstrDef::I64Const(i64_to_bitpattern(value)),
        Operator::GlobalGet {
            global_index,
        } => ConstInstrDef::GlobalGet(global_index),
        other => {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "unsupported {context}: expected i32.const, i64.const or global.get, found {}",
                    opcode_name(&other)
                ),
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
    Ok(ConstExprDef {
        instrs: vec![init],
    })
}

pub fn lower_func_sig(func: &FuncType) -> syn::Result<FuncSig> {
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
        results,
    })
}

pub fn lower_val_type(ty: ValType) -> syn::Result<Val> {
    match ty {
        ValType::I32 => Ok(Val::I32),
        ValType::I64 => Ok(Val::I64),
        ValType::F32 => Ok(Val::F32),
        ValType::F64 => Ok(Val::F64),
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

fn parse_memarg(memarg: wasmparser::MemArg, opcode: &str) -> syn::Result<MemArgDef> {
    let offset = u32::try_from(memarg.offset).map_err(|_| {
        Error::new(
            Span::call_site(),
            format!("opcode {opcode}: offset {} exceeds u32", memarg.offset),
        )
    })?;
    let align = u8::try_from(memarg.align).map_err(|_| {
        Error::new(
            Span::call_site(),
            format!("opcode {opcode}: align {} exceeds u8", memarg.align),
        )
    })?;
    Ok(MemArgDef {
        memory_index: memarg.memory,
        align,
        offset,
    })
}

fn i32_to_bitpattern(value: i32) -> u32 {
    value as u32
}

fn i64_to_bitpattern(value: i64) -> u64 {
    value as u64
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
        Operator::GlobalGet {
            ..
        } => "global.get",
        Operator::GlobalSet {
            ..
        } => "global.set",
        Operator::MemoryGrow {
            ..
        } => "memory.grow",
        Operator::CallIndirect {
            ..
        } => "call_indirect",
        Operator::BrTable {
            ..
        } => "br_table",
        Operator::RefNull {
            ..
        } => "ref.null",
        Operator::Unreachable => "unreachable",
        _ => "unknown opcode",
    }
}
