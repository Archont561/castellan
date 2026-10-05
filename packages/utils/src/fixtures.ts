/**
 * Test fixtures for `bun test`, the TypeScript mirror of rstest's `#[fixture]`.
 *
 * A fixture is a value the suite needs set up before a test (and torn down
 * after it): a scripted transport, a built WASM artifact, a temp directory.
 * Writing that wiring by hand every time is how suites end up with three
 * subtly different one-off `beforeEach` blocks; this helper expresses the
 * two lifecycles a fixture actually has:
 *
 * - `"test"` — a fresh value per test (`beforeEach`/`afterEach`). The
 *   default, because isolation is the safe choice: nothing a test does to
 *   its fixture can leak into the next test.
 * - `"file"` — one value for the whole file (`beforeAll`/`afterAll`). For
 *   expensive, immutable setup — a WASM artifact, a compiled module — where
 *   per-test freshness buys nothing and per-test cost is real.
 *
 * Call `createFixture` at suite scope (a describe body or module top level,
 * where bun accepts `beforeAll` and friends); it registers its hooks there
 * and hands back a getter that only works inside the wired lifecycle:
 *
 * One caveat, learned the hard way: "where the hooks are registered" is the
 * *test file* that evaluates the call, and an imported module is evaluated
 * once no matter how many test files import it. So a fixture defined at the
 * top level of a shared `support.ts` and exported as a value wires its whole
 * lifecycle into whichever file imported that module first, and every other
 * file reads it before setup and throws. Share the *factory* and let each
 * test file call it — the classes and helpers stay shared, only the
 * registration is per file:
 *
 * ```ts
 * // ./support.ts — shared, but a factory, not a fixture
 * export function scriptedClient() {
 *   return createFixture(() => new TestClient(new ScriptedTransport()));
 * }
 *
 * // ./client.test.ts
 * const client = scriptedClient();
 * ```
 *
 * ```ts
 * const wasm = createFixture(buildArtifact, undefined, "file");
 *
 * test("speaks the protocol version", () => {
 *   expect(wasm().protocolVersion()).toBe(1);
 * });
 * ```
 *
 * The getter throws — rather than handing back `undefined` — when it is
 * called before setup has run, so a fixture wired into the wrong scope
 * fails loudly at the call site instead of silently in every assertion.
 */
import { afterAll, afterEach, beforeAll, beforeEach } from "bun:test";

/** Create the fixture's value. May be async. */
export type Setup<T> = () => T | Promise<T>;

/** Dispose of the fixture's value. Receives what `Setup` produced. */
export type Teardown<T> = (value: T) => void | Promise<void>;

/** How long a fixture lives: one test, or the whole file. */
export type FixtureScope = "file" | "test";

/** The accessor `createFixture` returns: call it inside a test. */
export type Fixture<T> = () => T;

/**
 * Register a fixture and return its accessor.
 *
 * @param setup    runs before the scope's tests (once per scope)
 * @param teardown runs after the scope's tests (once per scope); skipped
 *                 when setup failed, so a half-built fixture is never torn
 *                 down as if it were whole
 * @param scope    `"test"` for a fresh value per test (the default),
 *                 `"file"` for one value shared by the whole file
 */
export function createFixture<T>(
  setup: Setup<T>,
  teardown: Teardown<T> | undefined = undefined,
  scope: FixtureScope = "test"
): Fixture<T> {
  // The holder's presence is the whole state machine: set once setup
  // finished building, cleared before teardown disposes — so "tear down
  // only what setup built" is a type-level fact, not a runtime flag.
  let holder: { readonly value: T } | undefined;

  const failEarly = (): never => {
    throw new Error(
      `fixture was read before its "${scope}" setup ran — read it inside a test, not at suite scope`
    );
  };

  const build = async (): Promise<void> => {
    holder = { value: await setup() };
  };
  const dispose = async (): Promise<void> => {
    // Only tear down what setup finished building.
    const built = holder;
    holder = undefined;
    if (built === undefined) {
      return;
    }
    await teardown?.(built.value);
  };

  if (scope === "file") {
    beforeAll(build);
    afterAll(dispose);
  } else {
    beforeEach(build);
    afterEach(dispose);
  }

  return () => {
    const built = holder;
    if (built === undefined) {
      return failEarly();
    }
    return built.value;
  };
}
