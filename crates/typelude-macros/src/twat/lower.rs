use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, LitStr};

use super::ir::{
    ConstExprDef, ConstInstrDef, DataSegmentDef, ElemSegmentDef, ExportDef, ExportKind, FuncSig,
    FunctionDef, GlobalDef, ImportDef, ImportKindDef, Instr, MemArgDef, MemoryDef, ModuleDef,
    TableDef, Val,
};

pub fn lower_func_space(module: &ModuleDef) -> syn::Result<TokenStream> {
    let types = lower_types(&module.types)?;
    let funcs = lower_functions(&module.functions)?;
    Ok(quote!(::typelude::wasm::WasmFuncSpace<#types, #funcs>))
}

pub fn lower_imports(imports: &[ImportDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(imports.len());
    for import in imports {
        lowered.push(lower_import(import)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_import(import: &ImportDef) -> syn::Result<TokenStream> {
    let module_name = LitStr::new(&import.module, Span::call_site());
    let field_name = LitStr::new(&import.field, Span::call_site());
    let kind = match &import.kind {
        ImportKindDef::Func(sig) => {
            let sig = lower_func_type(sig)?;
            quote!(::typelude::wasm::ImportFunc<#sig>)
        },
        ImportKindDef::Global {
            mutable,
            value_type,
        } => {
            let mutability = if *mutable {
                quote!(::typelude::wasm::GlobalMut)
            } else {
                quote!(::typelude::wasm::GlobalConst)
            };
            let value_type = lower_value_type(*value_type);
            quote!(::typelude::wasm::ImportGlobal<#mutability, #value_type>)
        },
        ImportKindDef::Memory {
            min,
            max,
        } => {
            let min = uint_type(*min as usize)?;
            let max = lower_limit(*max)?;
            quote!(::typelude::wasm::ImportMemory<#min, #max>)
        },
        ImportKindDef::Table {
            min,
            max,
        } => {
            let min = uint_type(*min as usize)?;
            let max = lower_limit(*max)?;
            quote!(::typelude::wasm::ImportTable<#min, #max>)
        },
    };
    Ok(quote!(
        ::typelude::wasm::WasmImport<
            ::typelude::wasm::tstr::TS!(#module_name),
            ::typelude::wasm::tstr::TS!(#field_name),
            #kind
        >
    ))
}

pub fn lower_types(types: &[FuncSig]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(types.len());
    for sig in types {
        lowered.push(lower_func_type(sig)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_functions(functions: &[FunctionDef]) -> syn::Result<TokenStream> {
    let mut funcs = Vec::with_capacity(functions.len());
    for function in functions {
        funcs.push(lower_function(function)?);
    }
    Ok(lower_list(funcs))
}

pub fn lower_function(function: &FunctionDef) -> syn::Result<TokenStream> {
    let program = lower_instrs(&function.body)?;
    let func_type = lower_func_type(&function.sig)?;
    let local_decls = lower_val_types(&function.local_decls)?;
    Ok(quote!(::typelude::wasm::WasmFunc<#func_type, #local_decls, #program>))
}

pub fn lower_func_type(sig: &FuncSig) -> syn::Result<TokenStream> {
    let params = lower_val_types(&sig.params)?;
    let results = lower_val_types(&sig.results)?;
    Ok(quote!(::typelude::wasm::WasmFuncType<#params, #results>))
}

pub fn lower_instrs(instructions: &[Instr]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(instructions.len());
    for instr in instructions {
        lowered.push(lower_instr(instr)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_instr(instr: &Instr) -> syn::Result<TokenStream> {
    Ok(match instr {
        Instr::Drop => quote!(::typelude::wasm::opcode::OpDrop),
        Instr::I32Const(value) => {
            let value = uint_type(*value as usize)?;
            quote!(::typelude::wasm::opcode::OpI32Const<#value>)
        },
        Instr::I64Const(value) => {
            let value = u64_type(*value)?;
            quote!(::typelude::wasm::opcode::OpI64Const<#value>)
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
        Instr::I64Add => quote!(::typelude::wasm::opcode::OpI64Add),
        Instr::I64Sub => quote!(::typelude::wasm::opcode::OpI64Sub),
        Instr::I64Eqz => quote!(::typelude::wasm::opcode::OpI64Eqz),
        Instr::I64Eq => quote!(::typelude::wasm::opcode::OpI64Eq),
        Instr::I64Ne => quote!(::typelude::wasm::opcode::OpI64Ne),
        Instr::I64LtS => quote!(::typelude::wasm::opcode::OpI64LtS),
        Instr::I64LtU => quote!(::typelude::wasm::opcode::OpI64LtU),
        Instr::I64GtS => quote!(::typelude::wasm::opcode::OpI64GtS),
        Instr::I64GtU => quote!(::typelude::wasm::opcode::OpI64GtU),
        Instr::I64LeS => quote!(::typelude::wasm::opcode::OpI64LeS),
        Instr::I64LeU => quote!(::typelude::wasm::opcode::OpI64LeU),
        Instr::I64GeS => quote!(::typelude::wasm::opcode::OpI64GeS),
        Instr::I64GeU => quote!(::typelude::wasm::opcode::OpI64GeU),
        Instr::I64And => quote!(::typelude::wasm::opcode::OpI64And),
        Instr::I64Or => quote!(::typelude::wasm::opcode::OpI64Or),
        Instr::I64Xor => quote!(::typelude::wasm::opcode::OpI64Xor),
        Instr::I64Shl => quote!(::typelude::wasm::opcode::OpI64Shl),
        Instr::I64ShrS => quote!(::typelude::wasm::opcode::OpI64ShrS),
        Instr::I64ShrU => quote!(::typelude::wasm::opcode::OpI64ShrU),
        Instr::I64Mul => quote!(::typelude::wasm::opcode::OpI64Mul),
        Instr::I64DivS => quote!(::typelude::wasm::opcode::OpI64DivS),
        Instr::I64DivU => quote!(::typelude::wasm::opcode::OpI64DivU),
        Instr::I64RemS => quote!(::typelude::wasm::opcode::OpI64RemS),
        Instr::I64RemU => quote!(::typelude::wasm::opcode::OpI64RemU),
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
        Instr::I32Load(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(::typelude::wasm::opcode::OpI32Load<#memarg>)
        },
        Instr::I32Store(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(::typelude::wasm::opcode::OpI32Store<#memarg>)
        },
        Instr::I32Load8U(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(::typelude::wasm::opcode::OpI32Load8U<#memarg>)
        },
        Instr::I32Store8(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(::typelude::wasm::opcode::OpI32Store8<#memarg>)
        },
        Instr::I64Load(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(::typelude::wasm::opcode::OpI64Load<#memarg>)
        },
        Instr::I64Store(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(::typelude::wasm::opcode::OpI64Store<#memarg>)
        },
        Instr::MemorySize(memory_index) => {
            let memory_index = uint_type(*memory_index as usize)?;
            quote!(::typelude::wasm::opcode::OpMemorySize<#memory_index>)
        },
        Instr::MemoryGrow(memory_index) => {
            let memory_index = uint_type(*memory_index as usize)?;
            quote!(::typelude::wasm::opcode::OpMemoryGrow<#memory_index>)
        },
    })
}

pub fn lower_val_types(values: &[Val]) -> syn::Result<TokenStream> {
    let mut items = Vec::with_capacity(values.len());
    for value in values {
        items.push(lower_value_type(*value));
    }
    Ok(lower_list(items))
}

pub fn lower_value_type(val: Val) -> TokenStream {
    match val {
        Val::I32 => quote!(::typelude::wasm::WasmI32Type),
        Val::I64 => quote!(::typelude::wasm::WasmI64Type),
    }
}

pub fn lower_memory_section(
    memory: Option<&MemoryDef>,
    data_segments: &[DataSegmentDef],
) -> syn::Result<TokenStream> {
    let data_segments = lower_data_segments(data_segments)?;
    match memory {
        Some(memory) => {
            let min = uint_type(memory.min as usize)?;
            let max = lower_limit(memory.max)?;
            Ok(quote!(
                ::typelude::wasm::WasmModuleMemory<
                    ::typelude::wasm::WasmMemoryDecl<#min, #max>,
                    #data_segments
                >
            ))
        },
        None => Ok(quote!(
            ::typelude::wasm::WasmModuleMemory<
                ::typelude::wasm::NoMemoryDecl,
                #data_segments
            >
        )),
    }
}

pub fn lower_data_segments(data_segments: &[DataSegmentDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(data_segments.len());
    for data_segment in data_segments {
        lowered.push(lower_data_segment(data_segment)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_data_segment(data_segment: &DataSegmentDef) -> syn::Result<TokenStream> {
    let offset = lower_const_expr(&data_segment.offset)?;
    let bytes = lower_bytes(&data_segment.bytes)?;
    Ok(quote!(::typelude::wasm::WasmDataSegment<#offset, #bytes>))
}

pub fn lower_tables_section(
    tables: &[TableDef],
    elem_segments: &[ElemSegmentDef],
) -> syn::Result<TokenStream> {
    let tables = lower_tables(tables)?;
    let elem_segments = lower_elem_segments(elem_segments)?;
    Ok(quote!(::typelude::wasm::WasmModuleTables<#tables, #elem_segments>))
}

pub fn lower_tables(tables: &[TableDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(tables.len());
    for table in tables {
        lowered.push(lower_table(table)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_table(table: &TableDef) -> syn::Result<TokenStream> {
    let min = uint_type(table.min as usize)?;
    let max = lower_limit(table.max)?;
    Ok(quote!(::typelude::wasm::WasmTableDecl<#min, #max>))
}

pub fn lower_elem_segments(elem_segments: &[ElemSegmentDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(elem_segments.len());
    for elem_segment in elem_segments {
        lowered.push(lower_elem_segment(elem_segment)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_elem_segment(elem_segment: &ElemSegmentDef) -> syn::Result<TokenStream> {
    let table_index = uint_type(elem_segment.table_index as usize)?;
    let offset = lower_const_expr(&elem_segment.offset)?;
    let func_indices = lower_u32_list(&elem_segment.func_indices)?;
    Ok(quote!(
        ::typelude::wasm::WasmElemSegment<#table_index, #offset, #func_indices>
    ))
}

pub fn lower_globals(globals: &[GlobalDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(globals.len());
    for global in globals {
        lowered.push(lower_global(global)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_global(global: &GlobalDef) -> syn::Result<TokenStream> {
    let mutability = if global.mutable {
        quote!(::typelude::wasm::GlobalMut)
    } else {
        quote!(::typelude::wasm::GlobalConst)
    };
    let init = lower_const_expr(&global.init)?;
    Ok(quote!(::typelude::wasm::WasmGlobalDecl<#mutability, #init>))
}

pub fn lower_const_expr(expr: &ConstExprDef) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(expr.instrs.len());
    for instr in &expr.instrs {
        lowered.push(match instr {
            ConstInstrDef::I32Const(value) => {
                let value = uint_type(*value as usize)?;
                quote!(::typelude::wasm::opcode::OpI32Const<#value>)
            },
            ConstInstrDef::I64Const(value) => {
                let value = u64_type(*value)?;
                quote!(::typelude::wasm::opcode::OpI64Const<#value>)
            },
            ConstInstrDef::GlobalGet(index) => {
                let index = uint_type(*index as usize)?;
                quote!(::typelude::wasm::opcode::OpGlobalGet<#index>)
            },
        });
    }
    let instrs = lower_list(lowered);
    Ok(quote!(::typelude::wasm::WasmConstExpr<#instrs>))
}

pub fn lower_exports(exports: &[ExportDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(exports.len());
    for export in exports {
        lowered.push(lower_export(export)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_export(export: &ExportDef) -> syn::Result<TokenStream> {
    let name = LitStr::new(&export.name, Span::call_site());
    let kind = match export.kind {
        ExportKind::Func(index) => {
            let index = uint_type(index as usize)?;
            quote!(::typelude::wasm::ExportFunc<#index>)
        },
        ExportKind::Global(index) => {
            let index = uint_type(index as usize)?;
            quote!(::typelude::wasm::ExportGlobal<#index>)
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

pub fn lower_start(start: Option<u32>) -> syn::Result<TokenStream> {
    match start {
        Some(index) => {
            let index = uint_type(index as usize)?;
            Ok(quote!(::typelude::wasm::StartFunc<#index>))
        },
        None => Ok(quote!(::typelude::wasm::NoStart)),
    }
}

pub fn lower_bytes(bytes: &[u32]) -> syn::Result<TokenStream> {
    lower_u32_list(bytes)
}

pub fn lower_u32_list(values: &[u32]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(values.len());
    for value in values {
        lowered.push(uint_type(*value as usize)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_limit(limit: Option<u32>) -> syn::Result<TokenStream> {
    match limit {
        Some(limit) => uint_type(limit as usize),
        None => Ok(quote!(::typelude::wasm::NoLimit)),
    }
}

pub fn lower_memarg(memarg: MemArgDef) -> syn::Result<TokenStream> {
    let memory_index = uint_type(memarg.memory_index as usize)?;
    let align = uint_type(usize::from(memarg.align))?;
    let offset = uint_type(memarg.offset as usize)?;
    Ok(quote!(::typelude::wasm::WasmMemArg<#memory_index, #align, #offset>))
}

pub fn lower_list(items: Vec<TokenStream>) -> TokenStream {
    items.into_iter().rev().fold(
        quote!(::typelude::wasm::TTerm),
        |tail, head| quote!(::typelude::wasm::TArr<#head, #tail>),
    )
}

fn u64_type(value: u64) -> syn::Result<TokenStream> {
    if value == u64::MAX {
        return Ok(quote!(
            ::typelude::typenum::operator_aliases::Or<
                <::typelude::typenum::Const<9223372036854775808> as ::typelude::typenum::ToUInt>::Output,
                <::typelude::typenum::Const<9223372036854775807> as ::typelude::typenum::ToUInt>::Output
            >
        ));
    }

    let value = usize::try_from(value).map_err(|_| {
        Error::new(
            Span::call_site(),
            format!("value {value} exceeds usize-backed typenum::Const support"),
        )
    })?;
    let literal = syn::LitInt::new(&format!("{value}usize"), Span::call_site());
    Ok(quote!(<::typelude::typenum::Const<#literal> as ::typelude::typenum::ToUInt>::Output))
}

fn uint_type(value: usize) -> syn::Result<TokenStream> {
    let lit = syn::LitInt::new(&value.to_string(), Span::call_site());
    Ok(quote!(<::typelude::typenum::Const<#lit> as ::typelude::typenum::ToUInt>::Output))
}
