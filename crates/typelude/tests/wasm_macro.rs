#![recursion_limit = "65536"]

use static_assertions::assert_type_eq_all;
use typelude::{
    typenum::{Const, ToUInt, U0, U1, U2, U3, U5, U7, U8, operator_aliases::Sum},
    wasm::{
        GlobalConst, GlobalMut, HostCall, HostCallResult, HostFuncBinding, HostGlobalBinding,
        HostMemoryBinding, HostTableBinding, InvokeExport, InvokeExportWithEnv, StateExportGlobal,
        StateExportMemory, StateExportTable, StateStack, TArr, TTerm, WasmF32, WasmGlobal,
        WasmHostEnv, WasmI32, WasmI32Type, WasmI64, WasmMemory, WasmTable,
    },
};

type NoArgs = TTerm;
type OneArg<A> = TArr<WasmI32<A>, TTerm>;
type TwoArgs<A, B> = TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>;
type U42 = <Const<42> as ToUInt>::Output;
type U247 = <Const<247> as ToUInt>::Output;
type U255 = <Const<255> as ToUInt>::Output;
type U258 = <Const<258> as ToUInt>::Output;
type U4660 = typelude::wasm::wasm_u32_bits_le!(0x34, 0x12, 0x00, 0x00);
type F32OneBits = typelude::wasm::wasm_u32_bits_le!(0x00, 0x00, 0x80, 0x3F);
type F64OneBits =
    typelude::wasm::wasm_u64_bits_le!(0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F);
type I64NegTwoBits =
    typelude::wasm::wasm_u64_bits_le!(0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF);

struct HostAdd;

impl<Store, A, B>
    HostCall<
        typelude::wasm::WasmFuncType<
            TArr<WasmI32Type, TArr<WasmI32Type, TTerm>>,
            TArr<WasmI32Type, TTerm>,
        >,
        Store,
        TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>,
    > for HostAdd
where
    A: core::ops::Add<B>,
{
    type Output = HostCallResult<Store, TArr<WasmI32<Sum<A, B>>, TTerm>>;
}

type CoreModule = typelude::twat! {
    module: r#"
        (module
          (func $add_internal (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add)
          (func $fib_impl (param i32) (result i32)
            local.get 0
            i32.eqz
            if
              i32.const 0
              return
            end
            local.get 0
            i32.const 1
            i32.sub
            i32.eqz
            if
              i32.const 1
              return
            end
            local.get 0
            i32.const 1
            i32.sub
            call $fib_impl
            local.get 0
            i32.const 2
            i32.sub
            call $fib_impl
            i32.add)
          (func (export "add") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add)
          (func (export "internal_call") (param i32) (result i32)
            local.get 0
            i32.const 2
            call $add_internal)
          (func (export "loop_countdown") (param i32) (result i32)
            block
              loop
                local.get 0
                i32.eqz
                br_if 1
                local.get 0
                i32.const 1
                i32.sub
                local.set 0
                br 0
              end
            end
            local.get 0)
          (func (export "if_select") (param i32) (result i32)
            (local i32)
            local.get 0
            if
              i32.const 10
              local.set 1
            else
              i32.const 20
              local.set 1
            end
            i32.const 30
            i32.const 40
            local.get 0
            select
            local.get 1
            i32.add)
          (func (export "zero_local") (result i32)
            (local i32)
            local.get 0)
          (func (export "fib") (param i32) (result i32)
            local.get 0
            call $fib_impl))
    "#,
};

type StateModule = typelude::twat! {
    module: r#"
        (module
          (memory (export "memory") 1 2)
          (global $g (export "g") (mut i32) (i32.const 2))
          (global $started (mut i32) (i32.const 0))
          (data (i32.const 16) "\2a")
          (func $start
            i32.const 7
            global.set $started
            i32.const 1
            i32.const 8
            i32.store8)
          (start $start)
          (func (export "memory_byte") (result i32)
            i32.const 0
            i32.const 255
            i32.store8
            i32.const 0
            i32.load8_u)
          (func (export "offset_grow") (result i32)
            i32.const 0
            i32.const 258
            i32.store offset=4
            i32.const 1
            memory.grow
            drop
            i32.const 5
            i32.load8_u)
          (func (export "data_read") (result i32)
            i32.const 16
            i32.load8_u)
          (func (export "global_update") (result i32)
            global.get $g
            i32.const 3
            i32.add
            global.set $g
            global.get $g)
          (func (export "started") (result i32)
            global.get $started))
    "#,
};

type DispatchModule = typelude::twat! {
    module: r#"
        (module
          (type $ret (func (result i32)))
          (table (export "table") 2 2 funcref)
          (table $t1 1 1 funcref)
          (elem (i32.const 0) $one $two)
          (elem (table $t1) (i32.const 0) func $three)
          (func $one (type $ret) (result i32)
            i32.const 1)
          (func $two (type $ret) (result i32)
            i32.const 2)
          (func $three (type $ret) (result i32)
            i32.const 3)
          (func (export "default_indirect") (result i32)
            i32.const 1
            call_indirect (type $ret))
          (func (export "explicit_indirect") (result i32)
            i32.const 0
            call_indirect $t1 (type $ret))
          (func (export "br_table") (param i32) (result i32)
            block
              block
                block
                  block
                    local.get 0
                    br_table 0 1 2 3
                  end
                  i32.const 10
                  return
                end
                i32.const 20
                return
              end
              i32.const 30
              return
            end
            i32.const 40)
        )
    "#,
};

type RuntimeParityModule = typelude::twat! {
    module: r#"
        (module
          (memory 1)
          (func (export "i32_parity") (result i32)
            i32.const 240
            i32.const 15
            i32.or
            i32.const 8
            i32.xor
            i32.const 1
            i32.shl
            i32.const 2
            i32.div_u)
          (func (export "wrap_i64") (result i32)
            i64.const 4294967298
            i32.wrap_i64)
          (func (export "partial_width_i32") (result i32)
            i32.const 1
            i32.const 4660
            i32.store16
            i32.const 1
            i32.load16_u)
          (func (export "partial_width_i64") (result i64)
            i32.const 0
            i64.const -2
            i64.store32
            i32.const 0
            i64.load32_s)
          (func (export "reinterpret_i64") (result i64)
            i64.const 4607182418800017408
            f64.reinterpret_i64
            i64.reinterpret_f64)
          (func (export "reinterpret_f32") (result f32)
            i32.const 1065353216
            f32.reinterpret_i32))
    "#,
};

type ImportFuncsGlobalsModule = typelude::twat! {
    module: r#"
        (module
          (import "host" "add" (func $add (param i32 i32) (result i32)))
          (import "host" "g" (global $g (mut i32)))
          (global $copy (export "copy") i32 (global.get $g))
          (func (export "imported_add") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            call $add)
          (func (export "imported_global") (result i32)
            global.get 0
            i32.const 2
            i32.add
            global.set 0
            global.get 0)
        )
    "#,
};

type ImportMemoryTableModule = typelude::twat! {
    module: r#"
        (module
          (type $ret (func (result i32)))
          (import "host" "memory" (memory $memory 1))
          (import "host" "table" (table $table 1 funcref))
          (export "memory" (memory $memory))
          (export "table_export" (table $table))
          (data (i32.const 0) "\2a")
          (elem (i32.const 0) func $seven)
          (func $seven (type $ret) (result i32)
            i32.const 7)
          (func (export "imported_memory") (result i32)
            i32.const 0
            i32.load8_u)
          (func (export "imported_table") (result i32)
            i32.const 0
            call_indirect (type $ret)))
    "#,
};

type FunctionEnv = WasmHostEnv<
    TArr<HostFuncBinding<typelude::tstr!("host"), typelude::tstr!("add"), HostAdd>, TTerm>,
    TTerm,
    TTerm,
    TTerm,
>;

type GlobalEnv = WasmHostEnv<
    TTerm,
    TArr<
        HostGlobalBinding<
            typelude::tstr!("host"),
            typelude::tstr!("g"),
            WasmGlobal<GlobalMut, WasmI32<U3>>,
        >,
        TTerm,
    >,
    TTerm,
    TTerm,
>;

type MemoryEnv = WasmHostEnv<
    TTerm,
    TTerm,
    TArr<
        HostMemoryBinding<
            typelude::tstr!("host"),
            typelude::tstr!("memory"),
            WasmMemory<U1, U1, TTerm>,
        >,
        TTerm,
    >,
    TTerm,
>;

type TableEnv = WasmHostEnv<
    TTerm,
    TTerm,
    TTerm,
    TArr<
        HostTableBinding<
            typelude::tstr!("host"),
            typelude::tstr!("table"),
            WasmTable<U1, U1, TTerm>,
        >,
        TTerm,
    >,
>;

#[test]
fn core_module_covers_basic_execution_paths() {
    type Add = InvokeExport<CoreModule, typelude::tstr!("add"), TwoArgs<U2, U3>>;
    type InternalCall = InvokeExport<CoreModule, typelude::tstr!("internal_call"), OneArg<U3>>;
    type LoopCountdown = InvokeExport<CoreModule, typelude::tstr!("loop_countdown"), OneArg<U3>>;
    type IfSelectTrue = InvokeExport<CoreModule, typelude::tstr!("if_select"), OneArg<U1>>;
    type IfSelectFalse = InvokeExport<CoreModule, typelude::tstr!("if_select"), OneArg<U0>>;
    type ZeroLocal = InvokeExport<CoreModule, typelude::tstr!("zero_local"), NoArgs>;
    type Fib =
        InvokeExport<CoreModule, typelude::tstr!("fib"), OneArg<<Const<6> as ToUInt>::Output>>;

    assert_type_eq_all!(<Add as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<InternalCall as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<LoopCountdown as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
    assert_type_eq_all!(
        <IfSelectTrue as StateStack>::Output,
        TArr<WasmI32<<Const<50> as ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(
        <IfSelectFalse as StateStack>::Output,
        TArr<WasmI32<<Const<60> as ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(<ZeroLocal as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
    assert_type_eq_all!(<Fib as StateStack>::Output, TArr<WasmI32<U8>, TTerm>);
}

#[test]
fn state_module_covers_memory_globals_and_start() {
    type MemoryByte = InvokeExport<StateModule, typelude::tstr!("memory_byte"), NoArgs>;
    type OffsetGrow = InvokeExport<StateModule, typelude::tstr!("offset_grow"), NoArgs>;
    type DataRead = InvokeExport<StateModule, typelude::tstr!("data_read"), NoArgs>;
    type GlobalUpdate = InvokeExport<StateModule, typelude::tstr!("global_update"), NoArgs>;
    type Started = InvokeExport<StateModule, typelude::tstr!("started"), NoArgs>;

    assert_type_eq_all!(<MemoryByte as StateStack>::Output, TArr<WasmI32<U255>, TTerm>);
    assert_type_eq_all!(<OffsetGrow as StateStack>::Output, TArr<WasmI32<U1>, TTerm>);
    assert_type_eq_all!(<DataRead as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
    assert_type_eq_all!(<GlobalUpdate as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<Started as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
    assert_type_eq_all!(
        <GlobalUpdate as StateExportGlobal<typelude::tstr!("g")>>::Output,
        WasmGlobal<GlobalMut, WasmI32<U5>>
    );
    type _MemoryExport1 = <MemoryByte as StateExportMemory<typelude::tstr!("memory")>>::Output;
    type _MemoryExport2 = <OffsetGrow as StateExportMemory<typelude::tstr!("memory")>>::Output;
}

#[test]
fn parity_module_covers_tables_dispatch_and_runtime_parity_ops() {
    type DefaultIndirect =
        InvokeExport<DispatchModule, typelude::tstr!("default_indirect"), NoArgs>;
    type ExplicitIndirect =
        InvokeExport<DispatchModule, typelude::tstr!("explicit_indirect"), NoArgs>;
    type BrTable0 = InvokeExport<DispatchModule, typelude::tstr!("br_table"), OneArg<U0>>;
    type BrTable1 = InvokeExport<DispatchModule, typelude::tstr!("br_table"), OneArg<U1>>;
    type BrTableDefault = InvokeExport<
        DispatchModule,
        typelude::tstr!("br_table"),
        OneArg<<Const<9> as ToUInt>::Output>,
    >;
    type I32Parity = InvokeExport<RuntimeParityModule, typelude::tstr!("i32_parity"), NoArgs>;
    type WrapI64 = InvokeExport<RuntimeParityModule, typelude::tstr!("wrap_i64"), NoArgs>;
    type PartialWidthI32 =
        InvokeExport<RuntimeParityModule, typelude::tstr!("partial_width_i32"), NoArgs>;
    type PartialWidthI64 =
        InvokeExport<RuntimeParityModule, typelude::tstr!("partial_width_i64"), NoArgs>;
    type ReinterpretI64 =
        InvokeExport<RuntimeParityModule, typelude::tstr!("reinterpret_i64"), NoArgs>;
    type ReinterpretF32 =
        InvokeExport<RuntimeParityModule, typelude::tstr!("reinterpret_f32"), NoArgs>;

    assert_type_eq_all!(<DefaultIndirect as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
    assert_type_eq_all!(<ExplicitIndirect as StateStack>::Output, TArr<WasmI32<U3>, TTerm>);
    assert_type_eq_all!(
        <BrTable0 as StateStack>::Output,
        TArr<WasmI32<<Const<10> as ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(
        <BrTable1 as StateStack>::Output,
        TArr<WasmI32<<Const<20> as ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(
        <BrTableDefault as StateStack>::Output,
        TArr<WasmI32<<Const<40> as ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(<I32Parity as StateStack>::Output, TArr<WasmI32<U247>, TTerm>);
    assert_type_eq_all!(<WrapI64 as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
    assert_type_eq_all!(<PartialWidthI32 as StateStack>::Output, TArr<WasmI32<U4660>, TTerm>);
    assert_type_eq_all!(
        <PartialWidthI64 as StateStack>::Output,
        TArr<WasmI64<I64NegTwoBits>, TTerm>
    );
    assert_type_eq_all!(<ReinterpretI64 as StateStack>::Output, TArr<WasmI64<F64OneBits>, TTerm>);
    assert_type_eq_all!(<ReinterpretF32 as StateStack>::Output, TArr<WasmF32<F32OneBits>, TTerm>);
    type _TableExport = <DefaultIndirect as StateExportTable<typelude::tstr!("table")>>::Output;
}

#[test]
fn imports_module_covers_imports_and_named_exports() {
    type ImportedAdd = InvokeExportWithEnv<
        ImportFuncsGlobalsModule,
        FunctionEnv,
        typelude::tstr!("imported_add"),
        TwoArgs<U2, U3>,
    >;
    type ImportedGlobal = InvokeExportWithEnv<
        ImportFuncsGlobalsModule,
        GlobalEnv,
        typelude::tstr!("imported_global"),
        NoArgs,
    >;
    type ImportedMemory = InvokeExportWithEnv<
        ImportMemoryTableModule,
        MemoryEnv,
        typelude::tstr!("imported_memory"),
        NoArgs,
    >;
    type ImportedTable = InvokeExportWithEnv<
        ImportMemoryTableModule,
        TableEnv,
        typelude::tstr!("imported_table"),
        NoArgs,
    >;

    assert_type_eq_all!(<ImportedAdd as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<ImportedGlobal as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<ImportedMemory as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
    assert_type_eq_all!(<ImportedTable as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
    assert_type_eq_all!(
        <ImportedGlobal as StateExportGlobal<typelude::tstr!("copy")>>::Output,
        WasmGlobal<GlobalConst, WasmI32<U3>>
    );
    type _ImportedMemoryExport =
        <ImportedMemory as StateExportMemory<typelude::tstr!("memory")>>::Output;
    type _ImportedTableExport =
        <ImportedTable as StateExportTable<typelude::tstr!("table_export")>>::Output;
}
