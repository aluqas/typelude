use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;

use super::{
    ir::{
        ConstExprDef, ConstInstrDef, DataSegmentDef, ElemSegmentDef, ExportDef, ExportKind,
        FuncSig, FunctionDef, GlobalDef, ImportDef, ImportKindDef, Instr, MemArgDef, MemoryDef,
        TableDef, Val,
    },
    validate::ValidatedModuleDef,
};

pub fn lower_func_space(module: &ValidatedModuleDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let types = lower_types(&module.types)?;
    let funcs = lower_functions(&module.functions)?;
    Ok(quote!(#p::WasmFuncSpace<#types, #funcs>))
}

pub fn lower_imports(imports: &[ImportDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(imports.len());
    for import in imports {
        lowered.push(lower_import(import)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_import(import: &ImportDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let module_name = LitStr::new(&import.module, Span::call_site());
    let field_name = LitStr::new(&import.field, Span::call_site());
    let kind = match &import.kind {
        ImportKindDef::Func(sig) => {
            let sig = lower_func_type(sig)?;
            quote!(#p::ImportFunc<#sig>)
        },
        ImportKindDef::Global {
            mutable,
            value_type,
        } => {
            let mutability = if *mutable {
                quote!(#p::GlobalMut)
            } else {
                quote!(#p::GlobalConst)
            };
            let value_type = lower_value_type(*value_type);
            quote!(#p::ImportGlobal<#mutability, #value_type>)
        },
        ImportKindDef::Memory {
            min,
            max,
        } => {
            let min = uint_type(*min as usize)?;
            let max = lower_limit(*max)?;
            quote!(#p::ImportMemory<#min, #max>)
        },
        ImportKindDef::Table {
            min,
            max,
        } => {
            let min = uint_type(*min as usize)?;
            let max = lower_limit(*max)?;
            quote!(#p::ImportTable<#min, #max>)
        },
    };
    Ok(quote!(
        #p::WasmImport<
            #p::tstr!(#module_name),
            #p::tstr!(#field_name),
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
    let p = quote!(::typelude::wasm::twat_prelude);
    let program = lower_instrs(&function.body)?;
    let func_type = lower_func_type(&function.sig)?;
    let local_decls = lower_val_types(&function.local_decls)?;
    Ok(quote!(#p::WasmFunc<#func_type, #local_decls, #program>))
}

pub fn lower_func_type(sig: &FuncSig) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let params = lower_val_types(&sig.params)?;
    let results = lower_val_types(&sig.results)?;
    Ok(quote!(#p::WasmFuncType<#params, #results>))
}

pub fn lower_instrs(instructions: &[Instr]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(instructions.len());
    for instr in instructions {
        lowered.push(lower_instr(instr)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_instr(instr: &Instr) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    Ok(match instr {
        Instr::Drop => quote!(#p::OpDrop),
        Instr::Nop => quote!(#p::OpNop),
        Instr::Unreachable => quote!(#p::OpUnreachable),
        Instr::I32Const(value) => {
            let value = uint_type(*value as usize)?;
            quote!(#p::OpI32Const<#value>)
        },
        Instr::I64Const(value) => {
            let value = u64_type(*value)?;
            quote!(#p::OpI64Const<#value>)
        },
        Instr::LocalGet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(#p::OpLocalGet<#index>)
        },
        Instr::LocalSet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(#p::OpLocalSet<#index>)
        },
        Instr::LocalTee(index) => {
            let index = uint_type(*index as usize)?;
            quote!(#p::OpLocalTee<#index>)
        },
        Instr::GlobalGet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(#p::OpGlobalGet<#index>)
        },
        Instr::GlobalSet(index) => {
            let index = uint_type(*index as usize)?;
            quote!(#p::OpGlobalSet<#index>)
        },
        Instr::I32Add => quote!(#p::OpI32Add),
        Instr::I32And => quote!(#p::OpI32And),
        Instr::I32Clz => quote!(#p::OpI32Clz),
        Instr::I32Sub => quote!(#p::OpI32Sub),
        Instr::I32Ctz => quote!(#p::OpI32Ctz),
        Instr::I32DivS => quote!(#p::OpI32DivS),
        Instr::I32DivU => quote!(#p::OpI32DivU),
        Instr::I32Eq => quote!(#p::OpI32Eq),
        Instr::I32Eqz => quote!(#p::OpI32Eqz),
        Instr::I32Extend8S => quote!(#p::OpI32Extend8S),
        Instr::I32Extend16S => quote!(#p::OpI32Extend16S),
        Instr::I32GeS => quote!(#p::OpI32GeS),
        Instr::I32GeU => quote!(#p::OpI32GeU),
        Instr::I32GtS => quote!(#p::OpI32GtS),
        Instr::I32GtU => quote!(#p::OpI32GtU),
        Instr::I32LeS => quote!(#p::OpI32LeS),
        Instr::I32LeU => quote!(#p::OpI32LeU),
        Instr::I32LtS => quote!(#p::OpI32LtS),
        Instr::I32LtU => quote!(#p::OpI32LtU),
        Instr::I32Mul => quote!(#p::OpI32Mul),
        Instr::I32Ne => quote!(#p::OpI32Ne),
        Instr::I32Or => quote!(#p::OpI32Or),
        Instr::I32Popcnt => quote!(#p::OpI32Popcnt),
        Instr::I32RemS => quote!(#p::OpI32RemS),
        Instr::I32RemU => quote!(#p::OpI32RemU),
        Instr::I32Rotl => quote!(#p::OpI32Rotl),
        Instr::I32Rotr => quote!(#p::OpI32Rotr),
        Instr::I32Shl => quote!(#p::OpI32Shl),
        Instr::I32ShrS => quote!(#p::OpI32ShrS),
        Instr::I32ShrU => quote!(#p::OpI32ShrU),
        Instr::I32WrapI64 => quote!(#p::OpI32WrapI64),
        Instr::I32Xor => quote!(#p::OpI32Xor),
        Instr::F32ReinterpretI32 => quote!(#p::OpF32ReinterpretI32),
        Instr::F64ReinterpretI64 => quote!(#p::OpF64ReinterpretI64),
        Instr::I64Add => quote!(#p::OpI64Add),
        Instr::I64And => quote!(#p::OpI64And),
        Instr::I64Clz => quote!(#p::OpI64Clz),
        Instr::I64Sub => quote!(#p::OpI64Sub),
        Instr::I64Ctz => quote!(#p::OpI64Ctz),
        Instr::I64Eqz => quote!(#p::OpI64Eqz),
        Instr::I64Eq => quote!(#p::OpI64Eq),
        Instr::I64ExtendI32S => quote!(#p::OpI64ExtendI32S),
        Instr::I64ExtendI32U => quote!(#p::OpI64ExtendI32U),
        Instr::I64Ne => quote!(#p::OpI64Ne),
        Instr::I64LtS => quote!(#p::OpI64LtS),
        Instr::I64LtU => quote!(#p::OpI64LtU),
        Instr::I64GtS => quote!(#p::OpI64GtS),
        Instr::I64GtU => quote!(#p::OpI64GtU),
        Instr::I64LeS => quote!(#p::OpI64LeS),
        Instr::I64LeU => quote!(#p::OpI64LeU),
        Instr::I64GeS => quote!(#p::OpI64GeS),
        Instr::I64GeU => quote!(#p::OpI64GeU),
        Instr::I64Or => quote!(#p::OpI64Or),
        Instr::I64Popcnt => quote!(#p::OpI64Popcnt),
        Instr::I64Xor => quote!(#p::OpI64Xor),
        Instr::I64Shl => quote!(#p::OpI64Shl),
        Instr::I64ShrS => quote!(#p::OpI64ShrS),
        Instr::I64ShrU => quote!(#p::OpI64ShrU),
        Instr::I64Rotl => quote!(#p::OpI64Rotl),
        Instr::I64Rotr => quote!(#p::OpI64Rotr),
        Instr::I64Mul => quote!(#p::OpI64Mul),
        Instr::I64DivS => quote!(#p::OpI64DivS),
        Instr::I64DivU => quote!(#p::OpI64DivU),
        Instr::I64RemS => quote!(#p::OpI64RemS),
        Instr::I64RemU => quote!(#p::OpI64RemU),
        Instr::I64ReinterpretF64 => quote!(#p::OpI64ReinterpretF64),
        Instr::Block(body) => {
            let body = lower_instrs(body)?;
            quote!(#p::OpBlock<#body>)
        },
        Instr::Loop(body) => {
            let body = lower_instrs(body)?;
            quote!(#p::OpLoop<#body>)
        },
        Instr::Br(depth) => {
            let depth = uint_type(*depth as usize)?;
            quote!(#p::OpBr<#depth>)
        },
        Instr::BrIf(depth) => {
            let depth = uint_type(*depth as usize)?;
            quote!(#p::OpBrIf<#depth>)
        },
        Instr::BrTable {
            targets,
            default,
        } => {
            let targets = lower_u32_list(targets)?;
            let default = uint_type(*default as usize)?;
            quote!(#p::OpBrTable<#targets, #default>)
        },
        Instr::If(then_body, else_body) => {
            let then_body = lower_instrs(then_body)?;
            let else_body = lower_instrs(else_body)?;
            quote!(#p::OpIf<#then_body, #else_body>)
        },
        Instr::Select => quote!(#p::OpSelect),
        Instr::Call(index) => {
            let index = uint_type(*index as usize)?;
            quote!(#p::OpCall<#index>)
        },
        Instr::CallIndirect {
            type_index,
            table_index,
        } => {
            let type_index = uint_type(*type_index as usize)?;
            let table_index = uint_type(*table_index as usize)?;
            quote!(#p::OpCallIndirect<#type_index, #table_index>)
        },
        Instr::Return => quote!(#p::OpReturn),
        Instr::I32Load(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Load<#memarg>)
        },
        Instr::I32Store(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Store<#memarg>)
        },
        Instr::I32Load8S(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Load8S<#memarg>)
        },
        Instr::I32Load8U(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Load8U<#memarg>)
        },
        Instr::I32Load16S(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Load16S<#memarg>)
        },
        Instr::I32Load16U(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Load16U<#memarg>)
        },
        Instr::I32Store8(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Store8<#memarg>)
        },
        Instr::I32Store16(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI32Store16<#memarg>)
        },
        Instr::I64Load8S(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load8S<#memarg>)
        },
        Instr::I64Load8U(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load8U<#memarg>)
        },
        Instr::I64Load16S(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load16S<#memarg>)
        },
        Instr::I64Load16U(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load16U<#memarg>)
        },
        Instr::I64Load32S(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load32S<#memarg>)
        },
        Instr::I64Load32U(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load32U<#memarg>)
        },
        Instr::I64Load(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Load<#memarg>)
        },
        Instr::I64Store8(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Store8<#memarg>)
        },
        Instr::I64Store16(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Store16<#memarg>)
        },
        Instr::I64Store32(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Store32<#memarg>)
        },
        Instr::I64Store(memarg) => {
            let memarg = lower_memarg(*memarg)?;
            quote!(#p::OpI64Store<#memarg>)
        },
        Instr::MemorySize(memory_index) => {
            let memory_index = uint_type(*memory_index as usize)?;
            quote!(#p::OpMemorySize<#memory_index>)
        },
        Instr::MemoryGrow(memory_index) => {
            let memory_index = uint_type(*memory_index as usize)?;
            quote!(#p::OpMemoryGrow<#memory_index>)
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
    let p = quote!(::typelude::wasm::twat_prelude);
    match val {
        Val::I32 => quote!(#p::WasmI32Type),
        Val::I64 => quote!(#p::WasmI64Type),
        Val::F32 => quote!(#p::WasmF32Type),
        Val::F64 => quote!(#p::WasmF64Type),
    }
}

pub fn lower_memory_section(
    memory: Option<&MemoryDef>,
    data_segments: &[DataSegmentDef],
) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let data_segments = lower_data_segments(data_segments)?;
    match memory {
        Some(memory) => {
            let min = uint_type(memory.min as usize)?;
            let max = lower_limit(memory.max)?;
            Ok(quote!(#p::WasmModuleMemory<#p::WasmMemoryDecl<#min, #max>, #data_segments>))
        },
        None => Ok(quote!(#p::WasmModuleMemory<#p::NoMemoryDecl, #data_segments>)),
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
    let p = quote!(::typelude::wasm::twat_prelude);
    let offset = lower_const_expr(&data_segment.offset)?;
    let bytes = lower_bytes(&data_segment.bytes)?;
    Ok(quote!(#p::WasmDataSegment<#offset, #bytes>))
}

pub fn lower_tables_section(
    tables: &[TableDef],
    elem_segments: &[ElemSegmentDef],
) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let tables = lower_tables(tables)?;
    let elem_segments = lower_elem_segments(elem_segments)?;
    Ok(quote!(#p::WasmModuleTables<#tables, #elem_segments>))
}

pub fn lower_tables(tables: &[TableDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(tables.len());
    for table in tables {
        lowered.push(lower_table(table)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_table(table: &TableDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let min = uint_type(table.min as usize)?;
    let max = lower_limit(table.max)?;
    Ok(quote!(#p::WasmTableDecl<#min, #max>))
}

pub fn lower_elem_segments(elem_segments: &[ElemSegmentDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(elem_segments.len());
    for elem_segment in elem_segments {
        lowered.push(lower_elem_segment(elem_segment)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_elem_segment(elem_segment: &ElemSegmentDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let table_index = uint_type(elem_segment.table_index as usize)?;
    let offset = lower_const_expr(&elem_segment.offset)?;
    let func_indices = lower_u32_list(&elem_segment.func_indices)?;
    Ok(quote!(#p::WasmElemSegment<#table_index, #offset, #func_indices>))
}

pub fn lower_globals(globals: &[GlobalDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(globals.len());
    for global in globals {
        lowered.push(lower_global(global)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_global(global: &GlobalDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let mutability = if global.mutable {
        quote!(#p::GlobalMut)
    } else {
        quote!(#p::GlobalConst)
    };
    let init = lower_const_expr(&global.init)?;
    Ok(quote!(#p::WasmGlobalDecl<#mutability, #init>))
}

pub fn lower_const_expr(expr: &ConstExprDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let mut lowered = Vec::with_capacity(expr.instrs.len());
    for instr in &expr.instrs {
        lowered.push(match instr {
            ConstInstrDef::I32Const(value) => {
                let value = uint_type(*value as usize)?;
                quote!(#p::OpI32Const<#value>)
            },
            ConstInstrDef::I64Const(value) => {
                let value = u64_type(*value)?;
                quote!(#p::OpI64Const<#value>)
            },
            ConstInstrDef::GlobalGet(index) => {
                let index = uint_type(*index as usize)?;
                quote!(#p::OpGlobalGet<#index>)
            },
        });
    }
    let instrs = lower_list(lowered);
    Ok(quote!(#p::WasmConstExpr<#instrs>))
}

pub fn lower_exports(exports: &[ExportDef]) -> syn::Result<TokenStream> {
    let mut lowered = Vec::with_capacity(exports.len());
    for export in exports {
        lowered.push(lower_export(export)?);
    }
    Ok(lower_list(lowered))
}

pub fn lower_export(export: &ExportDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let name = LitStr::new(&export.name, Span::call_site());
    let kind = match export.kind {
        ExportKind::Func(index) => {
            let index = uint_type(index as usize)?;
            quote!(#p::ExportFunc<#index>)
        },
        ExportKind::Global(index) => {
            let index = uint_type(index as usize)?;
            quote!(#p::ExportGlobal<#index>)
        },
        ExportKind::Memory => quote!(#p::ExportMemory),
        ExportKind::Table(index) => {
            let index = uint_type(index as usize)?;
            quote!(#p::ExportTable<#index>)
        },
    };
    Ok(quote!(#p::WasmExport<#p::tstr!(#name), #kind>))
}

pub fn lower_start(start: Option<u32>) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    match start {
        Some(index) => {
            let index = uint_type(index as usize)?;
            Ok(quote!(#p::StartFunc<#index>))
        },
        None => Ok(quote!(#p::NoStart)),
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
        None => Ok(quote!(::typelude::wasm::twat_prelude::NoLimit)),
    }
}

pub fn lower_memarg(memarg: MemArgDef) -> syn::Result<TokenStream> {
    let p = quote!(::typelude::wasm::twat_prelude);
    let memory_index = uint_type(memarg.memory_index as usize)?;
    let align = uint_type(usize::from(memarg.align))?;
    let offset = uint_type(memarg.offset as usize)?;
    Ok(quote!(#p::WasmMemArg<#memory_index, #align, #offset>))
}

pub fn lower_list(items: Vec<TokenStream>) -> TokenStream {
    if items.is_empty() {
        quote!(::typelude::wasm::twat_prelude::TTerm)
    } else {
        quote!(::typelude::wasm::twat_prelude::tarr![#(#items),*])
    }
}

fn u64_type(value: u64) -> syn::Result<TokenStream> {
    let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_le_bytes();
    let b0 = syn::LitInt::new(&b0.to_string(), Span::call_site());
    let b1 = syn::LitInt::new(&b1.to_string(), Span::call_site());
    let b2 = syn::LitInt::new(&b2.to_string(), Span::call_site());
    let b3 = syn::LitInt::new(&b3.to_string(), Span::call_site());
    let b4 = syn::LitInt::new(&b4.to_string(), Span::call_site());
    let b5 = syn::LitInt::new(&b5.to_string(), Span::call_site());
    let b6 = syn::LitInt::new(&b6.to_string(), Span::call_site());
    let b7 = syn::LitInt::new(&b7.to_string(), Span::call_site());
    Ok(quote!(
        ::typelude::wasm::twat_prelude::wasm_u64_bits_le!(#b0, #b1, #b2, #b3, #b4, #b5, #b6, #b7)
    ))
}

fn uint_type(value: usize) -> syn::Result<TokenStream> {
    if u32::try_from(value).is_ok() {
        let [b0, b1, b2, b3] = (value as u32).to_le_bytes();
        let b0 = syn::LitInt::new(&b0.to_string(), Span::call_site());
        let b1 = syn::LitInt::new(&b1.to_string(), Span::call_site());
        let b2 = syn::LitInt::new(&b2.to_string(), Span::call_site());
        let b3 = syn::LitInt::new(&b3.to_string(), Span::call_site());
        Ok(quote!(::typelude::wasm::twat_prelude::wasm_u32_bits_le!(#b0, #b1, #b2, #b3)))
    } else {
        u64_type(value as u64)
    }
}
