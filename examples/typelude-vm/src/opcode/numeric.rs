use paste::paste;

macro_rules! define_numeric_ops {
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

define_numeric_ops!(Add, Sub, Eq, Neq, Lt, Gt, Not, And, Or);
