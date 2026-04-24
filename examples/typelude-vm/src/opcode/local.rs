use core::marker::PhantomData;

use paste::paste;

macro_rules! define_local_ops {
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

define_local_ops!(Let, DropLocal);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGetLocal<Idx>(pub PhantomData<Idx>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSetLocal<Idx>(pub PhantomData<Idx>);
