#[derive(Clone)]
pub struct FuncSig {
    pub params: Vec<Val>,
    pub results: Vec<Val>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Val {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Clone)]
pub struct FunctionDef {
    pub sig: FuncSig,
    pub local_decls: Vec<Val>,
    pub body: Vec<Instr>,
}

#[derive(Clone)]
pub enum Instr {
    Drop,
    Nop,
    Unreachable,
    I32Const(u32),
    I64Const(u64),
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    I32Add,
    I32And,
    I32Clz,
    I32Sub,
    I32Ctz,
    I32DivS,
    I32DivU,
    I32Eq,
    I32Eqz,
    I32Extend8S,
    I32Extend16S,
    I32GeS,
    I32GeU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32LtS,
    I32LtU,
    I32Mul,
    I32Ne,
    I32Or,
    I32Popcnt,
    I32RemS,
    I32RemU,
    I32Rotl,
    I32Rotr,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32WrapI64,
    I32Xor,
    F32ReinterpretI32,
    F64ReinterpretI64,
    I64Add,
    I64And,
    I64Clz,
    I64Sub,
    I64Ctz,
    I64Eqz,
    I64Eq,
    I64ExtendI32S,
    I64ExtendI32U,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    I64Or,
    I64Popcnt,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64ReinterpretF64,
    Block(Vec<Instr>),
    Loop(Vec<Instr>),
    Br(u32),
    BrIf(u32),
    BrTable {
        targets: Vec<u32>,
        default: u32,
    },
    If(Vec<Instr>, Vec<Instr>),
    Select,
    Call(u32),
    CallIndirect {
        type_index: u32,
        table_index: u32,
    },
    Return,
    I32Load(MemArgDef),
    I32Store(MemArgDef),
    I32Load8S(MemArgDef),
    I32Load8U(MemArgDef),
    I32Load16S(MemArgDef),
    I32Load16U(MemArgDef),
    I32Store8(MemArgDef),
    I32Store16(MemArgDef),
    I64Load8S(MemArgDef),
    I64Load8U(MemArgDef),
    I64Load16S(MemArgDef),
    I64Load16U(MemArgDef),
    I64Load32S(MemArgDef),
    I64Load32U(MemArgDef),
    I64Load(MemArgDef),
    I64Store8(MemArgDef),
    I64Store16(MemArgDef),
    I64Store32(MemArgDef),
    I64Store(MemArgDef),
    MemorySize(u32),
    MemoryGrow(u32),
}

pub struct ModuleDef {
    pub imports: Vec<ImportDef>,
    pub types: Vec<FuncSig>,
    pub functions: Vec<FunctionDef>,
    pub memory: Option<MemoryDef>,
    pub tables: Vec<TableDef>,
    pub globals: Vec<GlobalDef>,
    pub data_segments: Vec<DataSegmentDef>,
    pub elem_segments: Vec<ElemSegmentDef>,
    pub exports: Vec<ExportDef>,
    pub start: Option<u32>,
}

pub struct ImportDef {
    pub module: String,
    pub field: String,
    pub kind: ImportKindDef,
}

#[derive(Clone)]
pub enum ImportKindDef {
    Func(FuncSig),
    Global {
        mutable: bool,
        value_type: Val,
    },
    Memory {
        min: u32,
        max: Option<u32>,
    },
    Table {
        min: u32,
        max: Option<u32>,
    },
}

pub struct MemoryDef {
    pub min: u32,
    pub max: Option<u32>,
}

pub struct TableDef {
    pub min: u32,
    pub max: Option<u32>,
}

pub struct GlobalDef {
    pub mutable: bool,
    pub init: ConstExprDef,
}

#[derive(Clone)]
pub enum ConstInstrDef {
    I32Const(u32),
    I64Const(u64),
    GlobalGet(u32),
}

#[derive(Clone)]
pub struct ConstExprDef {
    pub instrs: Vec<ConstInstrDef>,
}

#[derive(Clone, Copy)]
pub struct MemArgDef {
    pub memory_index: u32,
    pub align: u8,
    pub offset: u32,
}

pub struct DataSegmentDef {
    pub offset: ConstExprDef,
    pub bytes: Vec<u32>,
}

pub struct ElemSegmentDef {
    pub table_index: u32,
    pub offset: ConstExprDef,
    pub func_indices: Vec<u32>,
}

pub struct ExportDef {
    pub name: String,
    pub kind: ExportKind,
}

pub enum ExportKind {
    Func(u32),
    Global(u32),
    Memory,
    Table(u32),
}

pub enum Terminator {
    End,
    Else,
}
