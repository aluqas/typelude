use wasmi::{Engine, Linker, Memory, Module, Store};

#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub result_i32: i32,
    pub memory_pages: Option<u32>,
    pub memory_prefix: Vec<u8>,
}

pub fn run_wat_snapshot(
    module_wat: &str,
    export: &str,
    args: &[i32],
) -> Result<RuntimeSnapshot, wasmi::Error> {
    let engine = Engine::default();
    let module = Module::new(&engine, module_wat)?;
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
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

    let (memory_pages, memory_prefix) = match instance.get_memory(&store, "memory") {
        Some(memory) => snapshot_memory(memory, &store),
        None => (None, Vec::new()),
    };

    Ok(RuntimeSnapshot {
        result_i32,
        memory_pages,
        memory_prefix,
    })
}

fn snapshot_memory(memory: Memory, store: &Store<()>) -> (Option<u32>, Vec<u8>) {
    let pages = u32::try_from(memory.size(store)).ok();
    let prefix_len = memory.data_size(store).min(16);
    let prefix = memory.data(store)[..prefix_len].to_vec();
    (pages, prefix)
}
