/**
 * The first of the two suites sharing ./fixture-support.ts. Its sibling is
 * fixtures-across-files-b.test.ts; together they prove that a shared fixture
 * *factory* stays wired in every file that calls it.
 *
 * If `createFixture` were called once inside the support module and exported
 * as a value, bun would attach that one set of hooks to whichever file
 * imported it first — this file or its sibling — and the other would fail
 * with "fixture was read before its setup ran". Two files is the minimum that
 * can observe the trap, so both exist and neither is a duplicate.
 *
 * No assertion names a specific build number: which file registers first is
 * bun's load order, not this repo's business. What each file claims is that
 * its own fixture is wired — setup ran, so the count moved — and that the
 * default per-test scope holds inside a file that got its hooks from a
 * factory rather than from its own `createFixture` call.
 */
import { expect, test } from "bun:test";

import { sharedFixture, timesBuilt } from "./fixture-support";

const fixture = sharedFixture();

let previous = 0;

test("this file's fixture from the shared factory is wired", () => {
  const { value } = fixture();
  expect(value).toBeGreaterThan(0);
  expect(timesBuilt()).toBeGreaterThanOrEqual(value);
  previous = value;
});

test("the next test gets its own build, so the read is a per-test instance", () => {
  const { value } = fixture();
  expect(value).toBeGreaterThan(previous);
});
