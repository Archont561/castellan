# TypeScript packages

These workspace packages are the shared TypeScript half of Castellan. They
have narrow ownership boundaries: generated wire types do not gain behavior,
transport-neutral client mechanics do not import a platform runtime, and Svelte
components do not construct native clients.

| Package | Purpose | README |
| --- | --- | --- |
| `core/` | Transport-independent RPC client mechanics | [Core](core/README.md) |
| `protocol/` | Generated TypeScript protocol surface | [Protocol](protocol/README.md) |
| `tauri/` | Shared desktop/mobile Tauri transport | [Tauri](tauri/README.md) |
| `ui/` | Shared Svelte components and component tests | [UI](ui/README.md) |
| `utils/` | Shared config, fixtures, and build/test presets | [Utils](utils/README.md) |
| `wasm/` | Browser wrapper around Rust-compiled shared logic | [WASM](wasm/README.md) |

All packages are private workspace packages. Use Pixi from the repository root
for installation and commands; see the root [`README`](../README.md) and
[`AGENTS.md`](../AGENTS.md) for the toolchain contract.
