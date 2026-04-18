use core::ops::Deref;

use proc_macro2::Span;
use syn::Error;

use super::ir::{FunctionDef, Instr, ModuleDef};

pub struct ValidatedModuleDef(ModuleDef);

impl Deref for ValidatedModuleDef {
    type Target = ModuleDef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn validate_module(module: ModuleDef) -> syn::Result<ValidatedModuleDef> {
    let memory_count = module
        .imports
        .iter()
        .filter(|import| matches!(import.kind, super::ir::ImportKindDef::Memory { .. }))
        .count()
        + usize::from(module.memory.is_some());

    if memory_count > 1 {
        return Err(Error::new(
            Span::call_site(),
            "unsupported section: multiple memories are not supported",
        ));
    }

    for function in &module.functions {
        validate_function(function)?;
    }

    Ok(ValidatedModuleDef(module))
}

fn validate_function(function: &FunctionDef) -> syn::Result<()> {
    validate_instrs(&function.body)
}

fn validate_instrs(instrs: &[Instr]) -> syn::Result<()> {
    for instr in instrs {
        match instr {
            Instr::Block(body) | Instr::Loop(body) => validate_instrs(body)?,
            Instr::If(then_body, else_body) => {
                validate_instrs(then_body)?;
                validate_instrs(else_body)?;
            },
            Instr::I32Load(memarg)
            | Instr::I32Store(memarg)
            | Instr::I32Load8S(memarg)
            | Instr::I32Load8U(memarg)
            | Instr::I32Load16S(memarg)
            | Instr::I32Load16U(memarg)
            | Instr::I32Store8(memarg)
            | Instr::I32Store16(memarg)
            | Instr::I64Load8S(memarg)
            | Instr::I64Load8U(memarg)
            | Instr::I64Load16S(memarg)
            | Instr::I64Load16U(memarg)
            | Instr::I64Load32S(memarg)
            | Instr::I64Load32U(memarg)
            | Instr::I64Load(memarg)
            | Instr::I64Store8(memarg)
            | Instr::I64Store16(memarg)
            | Instr::I64Store32(memarg)
            | Instr::I64Store(memarg) => {
                validate_memory_index(memarg.memory_index, opcode_name(instr))?
            },
            Instr::MemorySize(memory_index) | Instr::MemoryGrow(memory_index) => {
                validate_memory_index(*memory_index, opcode_name(instr))?;
            },
            _ => {},
        }
    }
    Ok(())
}

fn validate_memory_index(memory_index: u32, opcode: &str) -> syn::Result<()> {
    if memory_index == 0 {
        Ok(())
    } else {
        Err(Error::new(
            Span::call_site(),
            format!("opcode {opcode}: memory index {memory_index} is not supported"),
        ))
    }
}

fn opcode_name(instr: &Instr) -> &'static str {
    match instr {
        Instr::I32Load(_) => "i32.load",
        Instr::I32Store(_) => "i32.store",
        Instr::I32Load8S(_) => "i32.load8_s",
        Instr::I32Load8U(_) => "i32.load8_u",
        Instr::I32Load16S(_) => "i32.load16_s",
        Instr::I32Load16U(_) => "i32.load16_u",
        Instr::I32Store8(_) => "i32.store8",
        Instr::I32Store16(_) => "i32.store16",
        Instr::I64Load8S(_) => "i64.load8_s",
        Instr::I64Load8U(_) => "i64.load8_u",
        Instr::I64Load16S(_) => "i64.load16_s",
        Instr::I64Load16U(_) => "i64.load16_u",
        Instr::I64Load32S(_) => "i64.load32_s",
        Instr::I64Load32U(_) => "i64.load32_u",
        Instr::I64Load(_) => "i64.load",
        Instr::I64Store8(_) => "i64.store8",
        Instr::I64Store16(_) => "i64.store16",
        Instr::I64Store32(_) => "i64.store32",
        Instr::I64Store(_) => "i64.store",
        Instr::MemorySize(_) => "memory.size",
        Instr::MemoryGrow(_) => "memory.grow",
        _ => "unknown opcode",
    }
}
