use core::marker::PhantomData;

use paste::paste;

macro_rules! define_simple_ops {
    ( $($name:ident),* ) => {
        paste! {
            $(
                #[doc = "Instruction: " $name]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
                pub struct [<Op $name>];
            )*
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpPush<Val>(pub PhantomData<Val>);

define_simple_ops!(
    Add, Sub, Dup, Swap, Drop, Pop, Eq, Neq, Lt, Gt, Not, And, Or, Load, Store, Return, Let,
    DropLocal
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpIf<Then, Else>(pub PhantomData<(Then, Else)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCall<Prog>(pub PhantomData<Prog>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpWhile<Cond, Body>(pub PhantomData<(Cond, Body)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGetLocal<Idx>(pub PhantomData<Idx>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSetLocal<Idx>(pub PhantomData<Idx>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpHostCall<Sig>(pub PhantomData<Sig>);
