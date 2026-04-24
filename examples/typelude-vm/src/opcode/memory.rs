use paste::paste;

macro_rules! define_memory_ops {
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

define_memory_ops!(Load, Store);
