# typelude-vm Semantics

- Stack top is the array head.
- Local and memory indices are 0-based from the array head.
- Binary operators consume the top value as `lhs` and the next value as `rhs`.
- `Store` consumes `value` first and `address` second.
- `Load` consumes the top address.
- `Call` preserves the stack, resets callee locals to `Nil`, and pushes a return frame with caller locals and continuation.
- `Return` restores caller locals and continuation from the top return frame.
- Trace records only executed instructions.
- `HostCall` is traced before suspension, and suspension stores the advanced state with the remaining program.
