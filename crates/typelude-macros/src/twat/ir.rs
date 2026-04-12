#[derive(Clone)]
pub struct FuncSig {
    pub params: Vec<Val>,
    pub results: Vec<Val>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Val {
    I32,
    I64,
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
    I32Const(u32),
    I64Const(u64),
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    I32Add,
    I32Sub,
    I32Eqz,
    I64Add,
    I64Sub,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    Block(Vec<Instr>),
    Loop(Vec<Instr>),
    Br(u32),
    BrIf(u32),
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
    I32Load8U(MemArgDef),
    I32Store8(MemArgDef),
    I64Load(MemArgDef),
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
