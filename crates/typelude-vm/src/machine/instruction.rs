//
// Instruction Set
//

// Defines the instruction types for the stack machine.
use core::marker::PhantomData;

use paste::paste;

/// Macro to define simple instructions with common derives.
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

// Marker trait for instructions could be added here if needed in future.

/// Pushes a value Val onto the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpPush<Val>(PhantomData<Val>);

// Define all simple operations
define_simple_ops!(
    Add, Sub, Dup, Swap, Drop, Pop, Eq, Neq, Lt, Gt, Not, And, Or, Load, Store, Return, Let,
    DropLocal
);

/// Conditional execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpIf<Then, Else>(PhantomData<(Then, Else)>);

/// Calls a subroutine (Program type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCall<Prog>(PhantomData<Prog>);

/// While loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpWhile<Cond, Body>(PhantomData<(Cond, Body)>);

/// Gets a local variable by De Bruijn index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGetLocal<Idx>(PhantomData<Idx>);

/// Sets a local variable by De Bruijn index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSetLocal<Idx>(PhantomData<Idx>);
