# typelude-std

- canonical ABI: `Eval`, `Apply`, `Op`
- shared capability traits: `core::{Not, Add, Len, Get, ...}`
- canonical operators: `core::ops::*`
- control-flow AST may use direct `Eval` semantics
- primitive-specific data types live outside `typelude-std`
