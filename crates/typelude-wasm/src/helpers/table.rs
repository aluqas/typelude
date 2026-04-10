use core::ops::Add;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eq, Lt};
use typenum::{B0, B1, U1, operator_aliases::Sum};

use crate::{
    helpers::i32::TrueBit,
    state::{NullFuncRef, TableEntry, WasmTable},
};

pub trait FindFuncRef<SlotIdx> {
    type Output;
}

impl<SlotIdx> FindFuncRef<SlotIdx> for TTerm {
    type Output = NullFuncRef;
}

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

impl<QueryIdx, EntryIdx, FuncRef, Tail> FindFuncRef<QueryIdx> for TArr<TableEntry<EntryIdx, FuncRef>, Tail>
where
    QueryIdx: Eq<EntryIdx>,
    <QueryIdx as Eq<EntryIdx>>::Output: FindFuncRefHelper<QueryIdx, FuncRef, Tail>,
{
    type Output =
        <<QueryIdx as Eq<EntryIdx>>::Output as FindFuncRefHelper<QueryIdx, FuncRef, Tail>>::Output;
}

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

pub trait TableWriteRef<SlotIdx, FuncRef> {
    type Output;
}

impl<Min, Max, Entries, SlotIdx, FuncRef> TableWriteRef<SlotIdx, FuncRef> for WasmTable<Min, Max, Entries>
where
    SlotIdx: Lt<Min>,
    <SlotIdx as Lt<Min>>::Output: TrueBit,
{
    type Output = WasmTable<Min, Max, TArr<TableEntry<SlotIdx, FuncRef>, Entries>>;
}

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
    type Output =
        <<Table as TableWriteRef<SlotIdx, FuncRef>>::Output as TableWriteRefs<Sum<SlotIdx, U1>, Tail>>::Output;
}
