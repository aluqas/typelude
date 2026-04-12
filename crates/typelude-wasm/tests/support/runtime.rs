use wasmi::{
    Engine, Global, Linker, Memory, MemoryType, Module, Mutability, Store, Table, TableType, Val,
    ValType,
};

#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub result_i32: i32,
    pub memory_pages: Option<u32>,
    pub memory_prefix: Vec<u8>,
    pub exported_global_i32: Option<i32>,
}

pub fn run_wat_snapshot(
    module_wat: &str,
    export: &str,
    args: &[i32],
) -> Result<RuntimeSnapshot, wasmi::Error> {
    run_wat_snapshot_with_env(module_wat, export, args, &RuntimeEnv::default())
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeGlobalImport {
    pub module: &'static str,
    pub field: &'static str,
    pub value: i32,
    pub mutable: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeMemoryImport {
    pub module: &'static str,
    pub field: &'static str,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeTableImport {
    pub module: &'static str,
    pub field: &'static str,
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeEnv {
    pub add_func: Option<(&'static str, &'static str)>,
    pub global: Option<RuntimeGlobalImport>,
    pub memory: Option<RuntimeMemoryImport>,
    pub table: Option<RuntimeTableImport>,
    pub observed_memory_export: Option<&'static str>,
    pub observed_global_export: Option<&'static str>,
}

pub fn run_wat_snapshot_with_env(
    module_wat: &str,
    export: &str,
    args: &[i32],
    env: &RuntimeEnv,
) -> Result<RuntimeSnapshot, wasmi::Error> {
    let engine = Engine::default();
    let module = Module::new(&engine, module_wat)?;
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    if let Some((module_name, field_name)) = env.add_func {
        linker.func_wrap(module_name, field_name, |lhs: i32, rhs: i32| -> i32 { lhs + rhs })?;
    }
    if let Some(global_import) = env.global {
        let global = Global::new(
            &mut store,
            Val::I32(global_import.value),
            if global_import.mutable {
                Mutability::Var
            } else {
                Mutability::Const
            },
        );
        linker.define(global_import.module, global_import.field, global)?;
    }
    if let Some(memory_import) = env.memory {
        let memory =
            Memory::new(&mut store, MemoryType::new(memory_import.min, memory_import.max))?;
        linker.define(memory_import.module, memory_import.field, memory)?;
    }
    if let Some(table_import) = env.table {
        let table = Table::new(
            &mut store,
            TableType::new(ValType::FuncRef, table_import.min, table_import.max),
            Val::default(ValType::FuncRef),
        )?;
        linker.define(table_import.module, table_import.field, table)?;
    }

    let instance = linker.instantiate_and_start(&mut store, &module)?;

    let result_i32 = match args {
        [] => instance.get_typed_func::<(), i32>(&store, export)?.call(&mut store, ())?,
        [arg0] => instance.get_typed_func::<i32, i32>(&store, export)?.call(&mut store, *arg0)?,
        [arg0, arg1] => instance
            .get_typed_func::<(i32, i32), i32>(&store, export)?
            .call(&mut store, (*arg0, *arg1))?,
        [arg0, arg1, arg2] => instance
            .get_typed_func::<(i32, i32, i32), i32>(&store, export)?
            .call(&mut store, (*arg0, *arg1, *arg2))?,
        _ => panic!("run_wat_snapshot supports up to 3 i32 args"),
    };

    let memory_export = env.observed_memory_export.unwrap_or("memory");
    let (memory_pages, memory_prefix) = match instance.get_memory(&store, memory_export) {
        Some(memory) => snapshot_memory(memory, &store),
        None => (None, Vec::new()),
    };
    let exported_global_i32 = env
        .observed_global_export
        .and_then(|name| instance.get_global(&store, name))
        .and_then(|global| global.get(&store).i32());

    Ok(RuntimeSnapshot {
        result_i32,
        memory_pages,
        memory_prefix,
        exported_global_i32,
    })
}

fn snapshot_memory(memory: Memory, store: &Store<()>) -> (Option<u32>, Vec<u8>) {
    let pages = u32::try_from(memory.size(store)).ok();
    let prefix_len = memory.data_size(store).min(16);
    let prefix = memory.data(store)[..prefix_len].to_vec();
    (pages, prefix)
}
