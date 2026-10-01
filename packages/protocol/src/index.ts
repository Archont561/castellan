/**
 * The TypeScript face of the Castellan protocol.
 *
 * Everything under `./generated` is derived from the Rust crate
 * `castellan-protocol` by `castellan-xtask` (`bun run codegen` from the repo
 * root). Do not edit it: a hand edit is a divergence between the wire the app
 * speaks and the types every client trusts, and it is overwritten on the next
 * generation anyway.
 *
 * This package contains types and one constant and nothing else. Behavior
 * lives in `@castellan/core`; UI lives in `@castellan/ui`. A protocol package
 * that can behave is a protocol package that drifts.
 */
export * from "./generated";
