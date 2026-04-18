mod input;
mod ir;
mod lower;
mod parse;
mod validate;

pub use input::WasmWatInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

pub fn expand(input: WasmWatInput) -> syn::Result<TokenStream> {
    let wasm = wat::parse_str(input.module.value())
        .map_err(|err| Error::new(input.module.span(), format!("WAT parse error: {err}")))?;
    let module = validate::validate_module(parse::parse_module(&wasm)?)?;
    let imports = lower::lower_imports(&module.imports)?;
    let funcs = lower::lower_func_space(&module)?;
    let memory = lower::lower_memory_section(module.memory.as_ref(), &module.data_segments)?;
    let tables = lower::lower_tables_section(&module.tables, &module.elem_segments)?;
    let globals = lower::lower_globals(&module.globals)?;
    let exports = lower::lower_exports(&module.exports)?;
    let start = lower::lower_start(module.start)?;

    Ok(quote!(
        ::typelude::wasm::WasmModule<
            #imports,
            #funcs,
            #memory,
            #tables,
            #globals,
            #exports,
            #start,
        >
    ))
}
