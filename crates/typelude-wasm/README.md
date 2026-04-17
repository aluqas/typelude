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
| `br_table`            | Y       | Y       |                                |
| `call`                | Y       | Y       |                                |
| `call_indirect`       | Y       | Y       |                                |
| `drop`                | Y       | Y       |                                |
| `else`                | -       | Y       | `if` 構文の一部として parse    |
| `f32.reinterpret_i32` | Y       | Y       | no float math                  |
| `f64.reinterpret_i64` | Y       | Y       | no float math                  |
| `global.get`          | Y       | Y       |                                |
| `global.set`          | Y       | Y       |                                |
| `i32.add`             | Y       | Y       |                                |
| `i32.and`             | Y       | Y       |                                |
| `i32.clz`             | Y       | Y       |                                |
| `i32.const`           | Y       | Y       | 負値も bitpattern 化して parse |
| `i32.ctz`             | Y       | Y       |                                |
| `i32.div_s`           | Y       | Y       |                                |
| `i32.div_u`           | Y       | Y       |                                |
| `i32.eq`              | Y       | Y       |                                |
| `i32.eqz`             | Y       | Y       |                                |
| `i32.extend_16_s`     | Y       | Y       |                                |
| `i32.extend_8_s`      | Y       | Y       |                                |
| `i32.ge_s`            | Y       | Y       |                                |
| `i32.ge_u`            | Y       | Y       |                                |
| `i32.gt_s`            | Y       | Y       |                                |
| `i32.gt_u`            | Y       | Y       |                                |
| `i32.le_s`            | Y       | Y       |                                |
| `i32.le_u`            | Y       | Y       |                                |
| `i32.load`            | Y       | Y       | memarg 保持                    |
| `i32.load16_s`        | Y       | Y       | memarg 保持                    |
| `i32.load16_u`        | Y       | Y       | memarg 保持                    |
| `i32.load8_s`         | Y       | Y       | memarg 保持                    |
| `i32.load8_u`         | Y       | Y       | memarg 保持                    |
| `i32.lt_s`            | Y       | Y       |                                |
| `i32.lt_u`            | Y       | Y       |                                |
| `i32.mul`             | Y       | Y       |                                |
| `i32.ne`              | Y       | Y       |                                |
| `i32.or`              | Y       | Y       |                                |
| `i32.popcnt`          | Y       | Y       |                                |
| `i32.reinterpret_f32` | N       | N       |                                |
| `i32.rem_s`           | Y       | Y       |                                |
| `i32.rem_u`           | Y       | Y       |                                |
| `i32.rotl`            | Y       | Y       |                                |
| `i32.rotr`            | Y       | Y       |                                |
| `i32.shl`             | Y       | Y       |                                |
| `i32.shr_s`           | Y       | Y       |                                |
| `i32.shr_u`           | Y       | Y       |                                |
| `i32.store`           | Y       | Y       | memarg 保持                    |
| `i32.store_16`        | Y       | Y       | memarg 保持                    |
| `i32.store_8`         | Y       | Y       | `i32.store8` として実装        |
| `i32.sub`             | Y       | Y       |                                |
| `i32.trunc_f32_s`     | N       | N       |                                |
| `i32.trunc_f64_s`     | N       | N       |                                |
| `i32.trunc_f64_u`     | N       | N       |                                |
| `i32.wrap_i64`        | Y       | Y       |                                |
| `i32.xor`             | Y       | Y       |                                |
| `i64.add`             | Y       | Y       |                                |
| `i64.and`             | Y       | Y       |                                |
| `i64.clz`             | Y       | Y       |                                |
| `i64.const`           | Y       | Y       |                                |
| `i64.ctz`             | Y       | Y       |                                |
| `i64.div_s`           | Y       | Y       |                                |
| `i64.div_u`           | Y       | Y       |                                |
| `i64.eq`              | Y       | Y       |                                |
| `i64.eqz`             | Y       | Y       |                                |
| `i64.extend_i32_s`    | Y       | Y       |                                |
| `i64.extend_i32_u`    | Y       | Y       |                                |
| `i64.ge_s`            | Y       | Y       |                                |
| `i64.ge_u`            | Y       | Y       |                                |
| `i64.gt_s`            | Y       | Y       |                                |
| `i64.gt_u`            | Y       | Y       |                                |
| `i64.le_s`            | Y       | Y       |                                |
| `i64.le_u`            | Y       | Y       |                                |
| `i64.load`            | Y       | Y       | memarg 保持                    |
| `i64.load16_s`        | Y       | Y       | memarg 保持                    |
| `i64.load16_u`        | Y       | Y       | memarg 保持                    |
| `i64.load32_s`        | Y       | Y       | memarg 保持                    |
| `i64.load32_u`        | Y       | Y       | memarg 保持                    |
| `i64.load8_s`         | Y       | Y       | memarg 保持                    |
| `i64.load8_u`         | Y       | Y       | memarg 保持                    |
| `i64.lt_s`            | Y       | Y       |                                |
| `i64.lt_u`            | Y       | Y       |                                |
| `i64.mul`             | Y       | Y       |                                |
| `i64.ne`              | Y       | Y       |                                |
| `i64.or`              | Y       | Y       |                                |
| `i64.popcnt`          | Y       | Y       |                                |
| `i64.reinterpret_f64` | Y       | Y       | no float math                  |
| `i64.rem_s`           | Y       | Y       |                                |
| `i64.rem_u`           | Y       | Y       |                                |
| `i64.rotl`            | Y       | Y       |                                |
| `i64.rotr`            | Y       | Y       |                                |
| `i64.shl`             | Y       | Y       |                                |
| `i64.shr_s`           | Y       | Y       |                                |
| `i64.shr_u`           | Y       | Y       |                                |
| `i64.store`           | Y       | Y       | memarg 保持                    |
| `i64.store_16`        | Y       | Y       | memarg 保持                    |
| `i64.store_32`        | Y       | Y       | memarg 保持                    |
| `i64.store_8`         | Y       | Y       | memarg 保持                    |
| `i64.sub`             | Y       | Y       |                                |
| `i64.xor`             | Y       | Y       |                                |
| `if`                  | Y       | Y       | block result なしのみ          |
| `local.get`           | Y       | Y       |                                |
| `local.set`           | Y       | Y       |                                |
| `local.tee`           | Y       | Y       |                                |
| `loop`                | Y       | Y       |                                |
| `memory.grow`         | Y       | Y       | memory index は現状 `0` 前提   |
| `memory.size`         | Y       | Y       | memory index は現状 `0` 前提   |
| `nop`                 | Y       | Y       |                                |
| `return`              | Y       | Y       |                                |
| `select`              | Y       | Y       | typed select は `i32/i64` のみ |
| `unreachable`         | P       | Y       | trap 契約は runtime 側へ委譲   |

Additionally, like many runtimes, a few "synthetic" instructions were created that are not according to any WebAssembly standard, but are sorta convenience features

| Synthetic Opcode  | Runtime | Notes                                  |
| ----------------- | ------- | -------------------------------------- |
| `synth.end_block` | Y       | internal only                          |
| `synth.end_func`  | Y       | internal only                          |
| `synth.end_loop`  | Y       | internal only                          |
| `synth.halt`      | N       | not present in current runtime surface |
