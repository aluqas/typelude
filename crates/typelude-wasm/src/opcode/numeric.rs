//! WebAssembly 数値命令 opcode。
//!
//! `i32` / `i64` の算術・比較・ビット演算と、
//! 一部の reinterpret / extend 系命令をマーカー型として定義します。
//! 各型の具体的な意味論は `instr::numeric` にあります。
//!
//! Stack effect は WASM の数値命令と同じです。
//! 二項算術・比較命令は `[T lhs, T rhs] -> [T|i32]`、
//! count / extend / reinterpret 系は `[T] -> [U]`、`drop` は `[T] -> []` です。
//! 比較命令の結果は WASM boolean として `WasmI32<0|1>` になります。

/// `drop`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpDrop;

/// `i32.add`。
///
/// Stack effect: `[i32, i32] -> [i32]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Add;

/// `i32.sub`。
///
/// Stack effect: `[i32, i32] -> [i32]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Sub;

/// `i32.eqz`。
///
/// Stack effect: `[i32] -> [i32 bool]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Eqz;

/// `i32.clz`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Clz;

/// `i32.ctz`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Ctz;

/// `i32.popcnt`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Popcnt;

/// `i32.eq`。
///
/// Stack effect: `[i32, i32] -> [i32 bool]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Eq;

/// `i32.ne`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Ne;

/// `i32.lt_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32LtS;

/// `i32.lt_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32LtU;

/// `i32.gt_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32GtS;

/// `i32.gt_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32GtU;

/// `i32.le_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32LeS;

/// `i32.le_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32LeU;

/// `i32.ge_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32GeS;

/// `i32.ge_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32GeU;

/// `i32.and`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32And;

/// `i32.or`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Or;

/// `i32.xor`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Xor;

/// `i32.shl`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Shl;

/// `i32.shr_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32ShrS;

/// `i32.shr_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32ShrU;

/// `i32.rotl`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Rotl;

/// `i32.rotr`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Rotr;

/// `i32.mul`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Mul;

/// `i32.div_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32DivS;

/// `i32.div_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32DivU;

/// `i32.rem_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32RemS;

/// `i32.rem_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32RemU;

/// `i32.extend8_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Extend8S;

/// `i32.extend16_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Extend16S;

/// `i32.wrap_i64`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32WrapI64;

/// `f32.reinterpret_i32`。
///
/// 現状は浮動小数演算ではなく bit-pattern の再解釈として使われます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpF32ReinterpretI32;

/// `f64.reinterpret_i64`。
///
/// 現状は浮動小数演算ではなく bit-pattern の再解釈として使われます。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpF64ReinterpretI64;

/// `i64.reinterpret_f64`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64ReinterpretF64;

/// `i64.add`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Add;

/// `i64.sub`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Sub;

/// `i64.eqz`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Eqz;

/// `i64.clz`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Clz;

/// `i64.ctz`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Ctz;

/// `i64.popcnt`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Popcnt;

/// `i64.eq`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Eq;

/// `i64.ne`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Ne;

/// `i64.lt_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64LtS;

/// `i64.lt_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64LtU;

/// `i64.gt_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64GtS;

/// `i64.gt_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64GtU;

/// `i64.le_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64LeS;

/// `i64.le_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64LeU;

/// `i64.ge_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64GeS;

/// `i64.ge_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64GeU;

/// `i64.and`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64And;

/// `i64.or`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Or;

/// `i64.xor`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Xor;

/// `i64.shl`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Shl;

/// `i64.shr_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64ShrS;

/// `i64.shr_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64ShrU;

/// `i64.rotl`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Rotl;

/// `i64.rotr`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Rotr;

/// `i64.mul`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Mul;

/// `i64.div_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64DivS;

/// `i64.div_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64DivU;

/// `i64.rem_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64RemS;

/// `i64.rem_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64RemU;

/// `i64.extend_i32_s`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64ExtendI32S;

/// `i64.extend_i32_u`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64ExtendI32U;
