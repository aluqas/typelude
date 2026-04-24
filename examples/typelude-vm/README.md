# typelude-vm Semantics

- Stack top is the array head.
- Local and memory indices are 0-based from the array head.
- Binary operators consume the top value as `lhs` and the next value as `rhs`.
- Canonical booleans are `True` and `False`.
- `Store` consumes `value` first and `address` second.
- `Load` consumes the top address.
- `Call` preserves the stack, resets callee locals to `Nil`, and pushes a return frame with caller locals and continuation.
- `Return` restores caller locals and continuation from the top return frame.
- Source trace records surface opcode dispatch.
- Core trace records lowered/core execution. `OpWhile` appears only in source trace.
- Trace is recorded before interpreting the step result.
- `StepContinue` commits the next state and recurses.
- `StepTrap` does not commit the failing step state.
- `StepSuspend` commits the advanced state and then yields.
- `HostCall` is traced before suspension, and suspension stores the advanced state with the remaining program.
- Resume pushes the host response onto the stack head and continues from the suspended state.
- Static well-formedness proofs reuse pure instruction semantics and do not replace runtime trap behavior.
