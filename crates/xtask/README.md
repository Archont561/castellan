# `castellan-xtask`

Typed repository automation that must understand Castellan's contract and
layout. It replaces fragile source-generating shell scripts with explicit Rust
subcommands that work from the repository root or any working directory.

## Commands

```console
$ pixi run xtask codegen
$ pixi run xtask version
```

`codegen` derives and writes the committed TypeScript protocol types,
face-scoped clients, native-host identity constants, and browser manifest
templates from the Rust protocol and extension host metadata. Review its output
as part of a contract change; never patch generated consumers by hand.

Every command accepts `--root <path>` when operating on another checkout. The
default derives the repository root from this crate's manifest rather than the
current working directory.

## Verify

```console
$ pixi run cargo nextest run -p castellan-xtask
$ pixi run cargo clippy -p castellan-xtask --all-targets -- -D warnings
$ pixi run codegen-check
```
