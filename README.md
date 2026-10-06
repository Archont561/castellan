# Offline sandbox (orphan branch)

Built 2026-10-06T19:46:32Z from commit `8193c58` for platform `linux-64`.
`pixi.lock` sha256 `1699016c38df42a2be7656ff39402969dd77a8d54bdc9e1b9faa3058698bf9c3`.

The verified self-bootstrap binary is stored at `.pixi-sandbox/tools/linux-64/pixi-sandbox`. The branch root intentionally contains documentation only.

| env | platform | packed | unpacked | files |
| --- | --- | ---: | ---: | ---: |
| `default` | linux-64 | 477.0 MiB | 2120.7 MiB | 58 |
| `shells` | linux-64 | 674.0 MiB | 2902.8 MiB | 187 |

Cargo dependencies: **489 crates**, 605.4 MiB (loose) from `Cargo.lock` sha256 `91815d4145c5…`; restore materialises them to `.pixi-sandbox/vendor/`. Built with cargo 1.98.1 (797e8a9bc 2026-08-05); rustc 1.98.1 (48a229cea 2026-09-01).

## Restore on the disconnected machine

```bash
./.pixi-sandbox/tools/linux-64/pixi-sandbox doctor --branch-location . --verify
./.pixi-sandbox/tools/linux-64/pixi-sandbox restore --branch-location . --output-path <project> --force
# then, from <project> with no network, use pixi as the sole entrypoint:
pixi install --frozen --offline
pixi run --frozen -- cargo build --offline
```

Every manifest blob is verified before it is written into the working tree.
