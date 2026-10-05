# Applications

Castellan has four top-level applications. The user-facing faces share one Rust
protocol but keep their platform integration at the edge: UI and generated
face clients live here; vault access, dispatch, IPC, and protocol rules live
under [`../crates/`](../crates/README.md).

| Application | Purpose | README |
| --- | --- | --- |
| `desktop/` | Desktop Tauri + SvelteKit face | [Desktop](desktop/README.md) |
| `mobile/` | Android/iOS Tauri + SvelteKit face | [Mobile](mobile/README.md) |
| `extension/` | Fob browser extension, built with WXT | [Extension](extension/README.md) |
| `docs/` | Astro + Starlight documentation site | [Docs site](docs/README.md) |

Run every command through Pixi from the repository root. The root
[`README`](../README.md) is the product and architecture overview; root
[`AGENTS.md`](../AGENTS.md) is the contribution contract.
