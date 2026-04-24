# Rust type-level stack machine

<https://github.com/MichiganTypeScript/typescript-types-only-wasm-runtime>

## Opcode Status

Legend:

- `Y`: implemented
- `P`: partial
- `N`: not implemented
- `-`: not a standalone runtime opcode

| Opcode                | Legacy Runtime | Checked Runtime | `twat!` | Notes                                                 |
| --------------------- | -------------- | --------------- | ------- | ----------------------------------------------------- |
| `block`               | Y              | Y               | Y       |                                                       |
| `br`                  | Y              | Y               | Y       |                                                       |
| `br_if`               | Y              | Y               | Y       |                                                       |
| `br_table`            | Y              | Y               | Y       |                                                       |
| `call`                | Y              | Y               | Y       |                                                       |
| `call_indirect`       | Y              | Y               | Y       | checked では null / OOB / type mismatch を trap 化    |
| `drop`                | Y              | Y               | Y       |                                                       |
| `else`                | -              | -               | Y       | `if` 構文の一部として parse                           |
| `f32.reinterpret_i32` | Y              | Y               | Y       | no float math                                         |
| `f64.reinterpret_i64` | Y              | Y               | Y       | no float math                                         |
| `global.get`          | Y              | Y               | Y       |                                                       |
| `global.set`          | Y              | Y               | Y       |                                                       |
| `i32.add`             | Y              | Y               | Y       |                                                       |
| `i32.and`             | Y              | Y               | Y       |                                                       |
| `i32.clz`             | Y              | Y               | Y       |                                                       |
| `i32.const`           | Y              | Y               | Y       | 負値も bitpattern 化して parse                        |
| `i32.ctz`             | Y              | Y               | Y       |                                                       |
| `i32.div_s`           | Y              | Y               | Y       |                                                       |
| `i32.div_u`           | Y              | Y               | Y       |                                                       |
| `i32.eq`              | Y              | Y               | Y       |                                                       |
| `i32.eqz`             | Y              | Y               | Y       |                                                       |
| `i32.extend_16_s`     | Y              | Y               | Y       |                                                       |
| `i32.extend_8_s`      | Y              | Y               | Y       |                                                       |
| `i32.ge_s`            | Y              | Y               | Y       |                                                       |
| `i32.ge_u`            | Y              | Y               | Y       |                                                       |
| `i32.gt_s`            | Y              | Y               | Y       |                                                       |
| `i32.gt_u`            | Y              | Y               | Y       |                                                       |
| `i32.le_s`            | Y              | Y               | Y       |                                                       |
| `i32.le_u`            | Y              | Y               | Y       |                                                       |
| `i32.load`            | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.load16_s`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.load16_u`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.load8_s`         | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.load8_u`         | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.lt_s`            | Y              | Y               | Y       |                                                       |
| `i32.lt_u`            | Y              | Y               | Y       |                                                       |
| `i32.mul`             | Y              | Y               | Y       |                                                       |
| `i32.ne`              | Y              | Y               | Y       |                                                       |
| `i32.or`              | Y              | Y               | Y       |                                                       |
| `i32.popcnt`          | Y              | Y               | Y       |                                                       |
| `i32.reinterpret_f32` | N              | N               | N       |                                                       |
| `i32.rem_s`           | Y              | Y               | Y       |                                                       |
| `i32.rem_u`           | Y              | Y               | Y       |                                                       |
| `i32.rotl`            | Y              | Y               | Y       |                                                       |
| `i32.rotr`            | Y              | Y               | Y       |                                                       |
| `i32.shl`             | Y              | Y               | Y       |                                                       |
| `i32.shr_s`           | Y              | Y               | Y       |                                                       |
| `i32.shr_u`           | Y              | Y               | Y       |                                                       |
| `i32.store`           | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.store_16`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i32.store_8`         | Y              | Y               | Y       | `i32.store8` として実装、checked では OOB を trap 化 |
| `i32.sub`             | Y              | Y               | Y       |                                                       |
| `i32.trunc_f32_s`     | N              | N               | N       |                                                       |
| `i32.trunc_f64_s`     | N              | N               | N       |                                                       |
| `i32.trunc_f64_u`     | N              | N               | N       |                                                       |
| `i32.wrap_i64`        | Y              | Y               | Y       |                                                       |
| `i32.xor`             | Y              | Y               | Y       |                                                       |
| `i64.add`             | Y              | Y               | Y       |                                                       |
| `i64.and`             | Y              | Y               | Y       |                                                       |
| `i64.clz`             | Y              | Y               | Y       |                                                       |
| `i64.const`           | Y              | Y               | Y       |                                                       |
| `i64.ctz`             | Y              | Y               | Y       |                                                       |
| `i64.div_s`           | Y              | Y               | Y       |                                                       |
| `i64.div_u`           | Y              | Y               | Y       |                                                       |
| `i64.eq`              | Y              | Y               | Y       |                                                       |
| `i64.eqz`             | Y              | Y               | Y       |                                                       |
| `i64.extend_i32_s`    | Y              | Y               | Y       |                                                       |
| `i64.extend_i32_u`    | Y              | Y               | Y       |                                                       |
| `i64.ge_s`            | Y              | Y               | Y       |                                                       |
| `i64.ge_u`            | Y              | Y               | Y       |                                                       |
| `i64.gt_s`            | Y              | Y               | Y       |                                                       |
| `i64.gt_u`            | Y              | Y               | Y       |                                                       |
| `i64.le_s`            | Y              | Y               | Y       |                                                       |
| `i64.le_u`            | Y              | Y               | Y       |                                                       |
| `i64.load`            | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.load16_s`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.load16_u`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.load32_s`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.load32_u`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.load8_s`         | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.load8_u`         | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.lt_s`            | Y              | Y               | Y       |                                                       |
| `i64.lt_u`            | Y              | Y               | Y       |                                                       |
| `i64.mul`             | Y              | Y               | Y       |                                                       |
| `i64.ne`              | Y              | Y               | Y       |                                                       |
| `i64.or`              | Y              | Y               | Y       |                                                       |
| `i64.popcnt`          | Y              | Y               | Y       |                                                       |
| `i64.reinterpret_f64` | Y              | Y               | Y       | no float math                                         |
| `i64.rem_s`           | Y              | Y               | Y       |                                                       |
| `i64.rem_u`           | Y              | Y               | Y       |                                                       |
| `i64.rotl`            | Y              | Y               | Y       |                                                       |
| `i64.rotr`            | Y              | Y               | Y       |                                                       |
| `i64.shl`             | Y              | Y               | Y       |                                                       |
| `i64.shr_s`           | Y              | Y               | Y       |                                                       |
| `i64.shr_u`           | Y              | Y               | Y       |                                                       |
| `i64.store`           | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.store_16`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.store_32`        | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.store_8`         | Y              | Y               | Y       | checked では OOB を trap 化、memarg は validate 対象  |
| `i64.sub`             | Y              | Y               | Y       |                                                       |
| `i64.xor`             | Y              | Y               | Y       |                                                       |
| `if`                  | Y              | Y               | Y       | block result なしのみ                                 |
| `local.get`           | Y              | Y               | Y       |                                                       |
| `local.set`           | Y              | Y               | Y       |                                                       |
| `local.tee`           | Y              | Y               | Y       |                                                       |
| `loop`                | Y              | Y               | Y       |                                                       |
| `memory.grow`         | Y              | Y               | Y       | memory index は validate で `0` のみに制限            |
| `memory.size`         | Y              | Y               | Y       | memory index は validate で `0` のみに制限            |
| `nop`                 | Y              | Y               | Y       |                                                       |
| `return`              | Y              | Y               | Y       |                                                       |
| `select`              | Y              | Y               | Y       | typed select は `i32/i64` のみ                        |
| `unreachable`         | P              | Y               | Y       | checked では `TrapUnreachable`、legacy は success-only |

Additionally, like many runtimes, a few "synthetic" instructions were created that are not according to any WebAssembly standard, but are sorta convenience features

| Synthetic Opcode  | Runtime | Notes                                  |
| ----------------- | ------- | -------------------------------------- |
| `synth.end_block` | Y       | internal only                          |
| `synth.end_func`  | Y       | internal only                          |
| `synth.end_loop`  | Y       | internal only                          |
| `synth.halt`      | N       | not present in current runtime surface |

## Frontend Pipeline

`twat!` is structured as `wat::parse_str -> parse_module -> validate_module -> lower_*`.

`twat!` column `Y` means the opcode is accepted by all three frontend phases:

- parse
- validate
- lower

The current validation pass fixes the following constraints explicitly:

- single-memory only
- memarg `memory_index == 0` only for `load` / `store` / `memory.size` / `memory.grow`

Additional unsupported forms are still rejected during parse:

- block / loop / if result types
- multi-value results
- passive / declarative data and elem segments
- unsupported reference types
- typed `select` outside `i32` / `i64`

## Runtime Surface

`Legacy Runtime` is the original success-only surface. Selected runtime failures still appear there as missing trait resolution or compile-fail edges.

`Checked Runtime` is the additive trap-aware surface:

- `RunChecked`
- `ModuleProgramRunChecked`
- `InvokeFuncChecked`
- `InvokeFuncCheckedWithEnv`
- `InvokeExportChecked`
- `InvokeExportCheckedWithEnv`

Checked traps currently exposed as type-level outcomes:

- `TrapUnreachable`
- `TrapCallIndirectNull`
- `TrapCallIndirectTableOob`
- `TrapCallIndirectTypeMismatch`
- `TrapMemoryOob`

In the checked runtime:

- `unreachable` traps with `TrapUnreachable`
- `call_indirect` traps on null table entries, table OOB, and type mismatch
- memory load/store traps on OOB
- `memory.size` remains infallible
- `memory.grow` remains value-producing rather than trapping
