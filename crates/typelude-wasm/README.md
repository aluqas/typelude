# Rust type-level stack machine

<https://github.com/MichiganTypeScript/typescript-types-only-wasm-runtime>

## Opcode Status

Legend:

- `Y`: implemented
- `P`: partial
- `N`: not implemented
- `-`: not a standalone runtime opcode

| Opcode                | Runtime | `twat!` | Notes                          |
| --------------------- | ------- | ------- | ------------------------------ |
| `block`               | Y       | Y       |                                |
| `br`                  | Y       | Y       |                                |
| `br_if`               | Y       | Y       |                                |
| `br_table`            | N       | N       |                                |
| `call`                | Y       | Y       |                                |
| `call_indirect`       | Y       | Y       |                                |
| `drop`                | Y       | Y       |                                |
| `else`                | -       | Y       | `if` 構文の一部として parse    |
| `f32.reinterpret_i32` | N       | N       |                                |
| `f64.reinterpret_i64` | N       | N       |                                |
| `global.get`          | Y       | Y       |                                |
| `global.set`          | Y       | Y       |                                |
| `i32.add`             | Y       | Y       |                                |
| `i32.and`             | Y       | N       | runtime 先行                   |
| `i32.clz`             | Y       | N       | runtime 先行                   |
| `i32.const`           | Y       | Y       | 負値も bitpattern 化して parse |
| `i32.ctz`             | Y       | N       | runtime 先行                   |
| `i32.div_s`           | Y       | N       | runtime 先行                   |
| `i32.div_u`           | Y       | N       | runtime 先行                   |
| `i32.eq`              | Y       | N       | runtime 先行                   |
| `i32.eqz`             | Y       | Y       |                                |
| `i32.extend_16_s`     | Y       | N       | runtime 先行                   |
| `i32.extend_8_s`      | Y       | N       | runtime 先行                   |
| `i32.ge_s`            | Y       | N       | runtime 先行                   |
| `i32.ge_u`            | Y       | N       | runtime 先行                   |
| `i32.gt_s`            | Y       | N       | runtime 先行                   |
| `i32.gt_u`            | Y       | N       | runtime 先行                   |
| `i32.le_s`            | Y       | N       | runtime 先行                   |
| `i32.le_u`            | Y       | N       | runtime 先行                   |
| `i32.load`            | Y       | Y       | memarg 保持                    |
| `i32.load16_s`        | Y       | N       | runtime 先行                   |
| `i32.load16_u`        | Y       | N       | runtime 先行                   |
| `i32.load8_s`         | Y       | N       | runtime 先行                   |
| `i32.load8_u`         | Y       | Y       | memarg 保持                    |
| `i32.lt_s`            | Y       | N       | runtime 先行                   |
| `i32.lt_u`            | Y       | N       | runtime 先行                   |
| `i32.mul`             | Y       | N       | runtime 先行                   |
| `i32.ne`              | Y       | N       | runtime 先行                   |
| `i32.or`              | Y       | N       | runtime 先行                   |
| `i32.popcnt`          | Y       | N       | runtime 先行                   |
| `i32.reinterpret_f32` | N       | N       |                                |
| `i32.rem_s`           | Y       | N       | runtime 先行                   |
| `i32.rem_u`           | Y       | N       | runtime 先行                   |
| `i32.rotl`            | Y       | N       | runtime 先行                   |
| `i32.rotr`            | Y       | N       | runtime 先行                   |
| `i32.shl`             | Y       | N       | runtime 先行                   |
| `i32.shr_s`           | Y       | N       | runtime 先行                   |
| `i32.shr_u`           | Y       | N       | runtime 先行                   |
| `i32.store`           | Y       | Y       | memarg 保持                    |
| `i32.store_16`        | Y       | N       | runtime 先行                   |
| `i32.store_8`         | Y       | Y       | `i32.store8` として実装        |
| `i32.sub`             | Y       | Y       |                                |
| `i32.trunc_f32_s`     | N       | N       |                                |
| `i32.trunc_f64_s`     | N       | N       |                                |
| `i32.trunc_f64_u`     | N       | N       |                                |
| `i32.wrap_i64`        | N       | N       |                                |
| `i32.xor`             | Y       | N       | runtime 先行                   |
| `i64.add`             | Y       | Y       |                                |
| `i64.and`             | Y       | Y       |                                |
| `i64.clz`             | Y       | N       | runtime 先行                   |
| `i64.const`           | Y       | Y       |                                |
| `i64.ctz`             | Y       | N       | runtime 先行                   |
| `i64.div_s`           | Y       | Y       |                                |
| `i64.div_u`           | Y       | Y       |                                |
| `i64.eq`              | Y       | Y       |                                |
| `i64.eqz`             | Y       | Y       |                                |
| `i64.extend_i32_s`    | Y       | N       | runtime 先行                   |
| `i64.extend_i32_u`    | Y       | N       | runtime 先行                   |
| `i64.ge_s`            | Y       | Y       |                                |
| `i64.ge_u`            | Y       | Y       |                                |
| `i64.gt_s`            | Y       | Y       |                                |
| `i64.gt_u`            | Y       | Y       |                                |
| `i64.le_s`            | Y       | Y       |                                |
| `i64.le_u`            | Y       | Y       |                                |
| `i64.load`            | Y       | Y       | memarg 保持                    |
| `i64.load16_s`        | Y       | N       | runtime 先行                   |
| `i64.load16_u`        | Y       | N       | runtime 先行                   |
| `i64.load32_s`        | Y       | N       | runtime 先行                   |
| `i64.load32_u`        | Y       | N       | runtime 先行                   |
| `i64.load8_s`         | Y       | N       | runtime 先行                   |
| `i64.load8_u`         | Y       | N       | runtime 先行                   |
| `i64.lt_s`            | Y       | Y       |                                |
| `i64.lt_u`            | Y       | Y       |                                |
| `i64.mul`             | Y       | Y       |                                |
| `i64.ne`              | Y       | Y       |                                |
| `i64.or`              | Y       | Y       |                                |
| `i64.popcnt`          | Y       | N       | runtime 先行                   |
| `i64.reinterpret_f64` | N       | N       |                                |
| `i64.rem_s`           | Y       | Y       |                                |
| `i64.rem_u`           | Y       | Y       |                                |
| `i64.rotl`            | Y       | N       | runtime 先行                   |
| `i64.rotr`            | Y       | N       | runtime 先行                   |
| `i64.shl`             | Y       | Y       |                                |
| `i64.shr_s`           | Y       | Y       |                                |
| `i64.shr_u`           | Y       | Y       |                                |
| `i64.store`           | Y       | Y       | memarg 保持                    |
| `i64.store_16`        | Y       | N       | runtime 先行                   |
| `i64.store_32`        | Y       | N       | runtime 先行                   |
| `i64.store_8`         | Y       | N       | runtime 先行                   |
| `i64.sub`             | Y       | Y       |                                |
| `i64.xor`             | Y       | Y       |                                |
| `if`                  | Y       | Y       | block result なしのみ          |
| `local.get`           | Y       | Y       |                                |
| `local.set`           | Y       | Y       |                                |
| `local.tee`           | Y       | Y       |                                |
| `loop`                | Y       | Y       |                                |
| `memory.grow`         | Y       | Y       | memory index は現状 `0` 前提   |
| `memory.size`         | Y       | Y       | memory index は現状 `0` 前提   |
| `nop`                 | Y       | N       | runtime 先行                   |
| `return`              | Y       | Y       |                                |
| `select`              | Y       | Y       | typed select は `i32/i64` のみ |
| `unreachable`         | P       | N       | opcode surface のみ、trap 契約は未固定 |

Additionally, like many runtimes, a few "synthetic" instructions were created that are not according to any WebAssembly standard, but are sorta convenience features

| Synthetic Opcode  | Runtime | Notes                                  |
| ----------------- | ------- | -------------------------------------- |
| `synth.end_block` | Y       | internal only                          |
| `synth.end_func`  | Y       | internal only                          |
| `synth.end_loop`  | Y       | internal only                          |
| `synth.halt`      | N       | not present in current runtime surface |
