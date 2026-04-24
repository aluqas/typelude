//! table 操作の helper 群。
//!
//! table は `TableEntry<SlotIdx, FuncIdx>` の疎なリストとして表現します。
//! 未書き込み slot は `NullFuncRef` として扱われ、checked `call_indirect` では
//! `TrapCallIndirectNull` へ変換されます。slot index が table limit 外の場合は
//! success-only lookup では trait 未解決、checked lookup では `TableSlotOob`
//! になります。

use core::ops::Add;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eq, Lt};
use typenum::{B0, B1, U1, operator_aliases::Sum};

use crate::{
    helpers::i32::TrueBit,
    state::{NullFuncRef, TableEntry, WasmTable},
};

/// table 読み出しが範囲外だったことを表す型。
pub struct TableSlotOob;

/// table entry 列から slot を探索する helper family。
pub trait FindFuncRef<SlotIdx> {
    type Output;
}

impl<SlotIdx> FindFuncRef<SlotIdx> for TTerm {
    type Output = NullFuncRef;
}

/// `FindFuncRef` の再帰処理用 helper。
pub trait FindFuncRefHelper<SlotIdx, FuncRef, Tail> {
    type Output;
}

impl<SlotIdx, FuncRef, Tail> FindFuncRefHelper<SlotIdx, FuncRef, Tail> for B1 {
    type Output = FuncRef;
}

impl<SlotIdx, FuncRef, Tail> FindFuncRefHelper<SlotIdx, FuncRef, Tail> for B0
where
    Tail: FindFuncRef<SlotIdx>,
{
    type Output = <Tail as FindFuncRef<SlotIdx>>::Output;
}

impl<QueryIdx, EntryIdx, FuncRef, Tail> FindFuncRef<QueryIdx>
    for TArr<TableEntry<EntryIdx, FuncRef>, Tail>
where
    QueryIdx: Eq<EntryIdx>,
    <QueryIdx as Eq<EntryIdx>>::Output: FindFuncRefHelper<QueryIdx, FuncRef, Tail>,
{
    type Output =
        <<QueryIdx as Eq<EntryIdx>>::Output as FindFuncRefHelper<QueryIdx, FuncRef, Tail>>::Output;
}

/// table から funcref を読み出す success-only helper。
pub trait TableReadRef<SlotIdx> {
    type Output;
}

impl<Min, Max, Entries, SlotIdx> TableReadRef<SlotIdx> for WasmTable<Min, Max, Entries>
where
    SlotIdx: Lt<Min>,
    <SlotIdx as Lt<Min>>::Output: TrueBit,
    Entries: FindFuncRef<SlotIdx>,
{
    type Output = <Entries as FindFuncRef<SlotIdx>>::Output;
}

/// table から funcref を読み出す checked helper。
pub trait TableReadRefChecked<SlotIdx> {
    type Output;
}

/// `TableReadRefChecked` の境界判定 helper。
pub trait TableReadRefCheckedHelper<SlotIdx, Entries> {
    type Output;
}

impl<SlotIdx, Entries> TableReadRefCheckedHelper<SlotIdx, Entries> for B0 {
    type Output = TableSlotOob;
}

impl<SlotIdx, Entries> TableReadRefCheckedHelper<SlotIdx, Entries> for B1
where
    Entries: FindFuncRef<SlotIdx>,
{
    type Output = <Entries as FindFuncRef<SlotIdx>>::Output;
}

impl<Min, Max, Entries, SlotIdx> TableReadRefChecked<SlotIdx> for WasmTable<Min, Max, Entries>
where
    SlotIdx: Lt<Min>,
    <SlotIdx as Lt<Min>>::Output: TableReadRefCheckedHelper<SlotIdx, Entries>,
{
    type Output =
        <<SlotIdx as Lt<Min>>::Output as TableReadRefCheckedHelper<SlotIdx, Entries>>::Output;
}

/// table の 1 slot に funcref を書き込む helper。
pub trait TableWriteRef<SlotIdx, FuncRef> {
    type Output;
}

impl<Min, Max, Entries, SlotIdx, FuncRef> TableWriteRef<SlotIdx, FuncRef>
    for WasmTable<Min, Max, Entries>
where
    SlotIdx: Lt<Min>,
    <SlotIdx as Lt<Min>>::Output: TrueBit,
{
    type Output = WasmTable<Min, Max, TArr<TableEntry<SlotIdx, FuncRef>, Entries>>;
}

/// 連続する table slot 群へ funcref 列を書き込む helper。
pub trait TableWriteRefs<SlotIdx, FuncRefs> {
    type Output;
}

impl<Table, SlotIdx> TableWriteRefs<SlotIdx, TTerm> for Table {
    type Output = Table;
}

impl<Table, SlotIdx, FuncRef, Tail> TableWriteRefs<SlotIdx, TArr<FuncRef, Tail>> for Table
where
    Table: TableWriteRef<SlotIdx, FuncRef>,
    SlotIdx: Add<U1>,
    <Table as TableWriteRef<SlotIdx, FuncRef>>::Output: TableWriteRefs<Sum<SlotIdx, U1>, Tail>,
{
    type Output = <<Table as TableWriteRef<SlotIdx, FuncRef>>::Output as TableWriteRefs<
        Sum<SlotIdx, U1>,
        Tail,
    >>::Output;
}
