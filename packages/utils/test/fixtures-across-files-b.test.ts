/**
 * The second of the two suites sharing ./fixture-support.ts — see
 * fixtures-across-files-a.test.ts for why there are two, what breaks if the
 * fixture is created in the support module instead of by its callers, and why
 * neither file asserts a specific build number.
 *
 * This file earns its place by being the one that loads second under the
 * mutation: the shared-instance shape wires the hooks here, and the sibling
 * is the one that throws.
 */
import { expect, test } from "bun:test";

import { sharedFixture, timesBuilt } from "./fixture-support";

const fixture = sharedFixture();

test("the sibling's fixture from the shared factory is wired too", () => {
  const value = fixture();
  expect(value.value).toBeGreaterThan(0);
  expect(timesBuilt()).toBeGreaterThanOrEqual(value.value);
});
