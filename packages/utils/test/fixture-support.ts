/**
 * A fixture definition shared by two test files, for the suite that pins how
 * `createFixture` composes with bun's per-file hook registration.
 *
 * This module deliberately exports a *factory*, not a fixture. Bun registers
 * `beforeEach`/`afterEach` against the test file that evaluates the call, and
 * an imported module is evaluated once no matter how many files import it —
 * so a `createFixture(...)` executed here would wire its whole lifecycle into
 * whichever file loaded this module first, and the other file would read the
 * fixture before setup and throw. Two suites import this factory; both stay
 * green only because each one registers its own hooks.
 */
import { createFixture } from "@/src/fixtures";

let built = 0;

/**
 * How many times setup ran across every file that called this factory. It is
 * shared state on purpose: a caller can only assert its own read is wired if
 * the count survives the module boundary.
 */
export function timesBuilt(): number {
  return built;
}

/** Register a fresh per-test fixture against whichever test file calls this. */
export function sharedFixture(): () => { value: number } {
  return createFixture(() => {
    built += 1;
    return { value: built };
  });
}
