/**
 * The reuse package: what every other package in this repo imports instead
 * of re-inventing — today that is the bun test fixtures and (through the
 * package exports, not this barrel) the TypeScript config bases under
 * `tsconfig/`.
 */

export type { Fixture, FixtureScope, Setup, Teardown } from "./fixtures";
export { createFixture } from "./fixtures";
