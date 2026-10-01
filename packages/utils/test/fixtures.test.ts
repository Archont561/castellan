/**
 * The fixture helper, proven on itself: the two lifecycles do what their
 * names promise, and the getter refuses to hand out a value that setup has
 * not produced yet.
 *
 * These tests are the contract every other package relies on when it
 * imports `createFixture` — if a change here breaks them, it breaks the
 * wasm artifact fixture and every future suite, not just this package.
 */
import { describe, expect, test } from "bun:test";

import { createFixture } from "@/src/fixtures";

describe("createFixture — test scope (the default)", () => {
  const events: string[] = [];
  let next = 0;

  const fresh = createFixture(
    () => {
      const mine = next++;
      events.push(`setup:${mine}`);
      return { mine };
    },
    (value) => {
      events.push(`teardown:${value.mine}`);
    }
  );

  test("hands the test a fresh value", () => {
    expect(fresh().mine).toBe(0);
    expect(events).toEqual(["setup:0"]);
  });

  test("and the next test a different one, after tearing the first down", () => {
    // If scope were per-file this would still be 0 and the events list
    // would have no teardown in it.
    expect(fresh().mine).toBe(1);
    expect(events).toEqual(["setup:0", "teardown:0", "setup:1"]);
  });
});

describe("createFixture — test scope isolation", () => {
  let value = 0;
  const box = createFixture(() => ({ value: value++ }));

  test("a test's mutation is its own", () => {
    box().value = 99;
    expect(box().value).toBe(99);
  });

  test("the next test starts from setup, not from the last test's mess", () => {
    expect(box().value).toBe(1);
  });
});

describe("createFixture — file scope", () => {
  let setups = 0;
  let teardowns = 0;

  const shared = createFixture(
    () => {
      setups += 1;
      return { payload: "built once" };
    },
    () => {
      teardowns += 1;
    },
    "file"
  );

  test("the first test sees the value (and is the only setup)", () => {
    expect(shared().payload).toBe("built once");
    expect(setups).toBe(1);
  });

  test("the second test sees the SAME value, not a second setup", () => {
    // Identity, not equality: one object, handed to every test.
    expect(shared()).toBe(shared());
    expect(setups).toBe(1);
    expect(teardowns).toBe(0);
  });

  test("mutations persist — the documented cost of file scope", () => {
    shared().payload = "mutated by test three";
    expect(setups).toBe(1);
  });

  test("…so the fourth test sees them; tests needing isolation pick test scope", () => {
    expect(shared().payload).toBe("mutated by test three");
    expect(setups).toBe(1);
  });
});

describe("createFixture — async setup and teardown", () => {
  const tornDown: number[] = [];

  const artifact = createFixture(
    async () => {
      await Promise.resolve();
      return Date.now();
    },
    async (value) => {
      await Promise.resolve();
      tornDown.push(value);
    }
  );

  test("awaits the setup before the test runs", () => {
    expect(Number.isFinite(artifact())).toBe(true);
  });

  test("and awaits the teardown between tests", () => {
    // The previous test's value was disposed of before this one began.
    expect(tornDown.length).toBe(1);
  });
});

describe("createFixture — teardown is optional", () => {
  const bare = createFixture(() => "nothing to clean up");

  test("a fixture without a teardown still builds and yields", () => {
    expect(bare()).toBe("nothing to clean up");
  });
});

describe("createFixture — reading before setup ran", () => {
  const lazy = createFixture(() => "never built yet");
  // Describe bodies run at collection time, before any hook: this read is
  // what a fixture wired into the wrong scope looks like.
  let earlyError: unknown;
  try {
    lazy();
  } catch (error) {
    earlyError = error;
  }

  test("fails loudly at the call site, not silently in assertions", () => {
    expect(earlyError).toBeInstanceOf(Error);
    expect((earlyError as Error).message).toMatch(/before its "test" setup ran/);
  });
});
