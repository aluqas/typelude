use core::marker::PhantomData;

use paste::paste;

macro_rules! define_stack_ops {
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

define_stack_ops!(Dup, Swap, Drop, Pop);
