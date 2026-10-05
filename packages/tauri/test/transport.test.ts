/**
 * The Tauri adapter's contract: one command, one argument, and the
 * envelope passed through untouched in both directions — plus any invoke
 * rejection normalized to the transport-independent DISCONNECTED error.
 *
 * The wiring is a per-test fixture (a scripted `invoke` under the
 * transport, the Tauri-side mirror of core's ScriptedTransport), and the
 * two pass-through claims are properties, because they hold for any
 * request the faces can send and any response the host can return — not
 * just the shapes a hand-written example thought to try.
 */
import { describe, expect, test } from "bun:test";
import type {
  EntrySummary,
  Event,
  NewEntry,
  RpcError,
  RpcMethod,
  RpcRequest,
  RpcResponse,
  RpcResult
} from "@castellan/protocol";
import { createFixture } from "@castellan/utils/fixtures";
import fc from "fast-check";

import {
  createTauriTransport,
  type TauriInvoke,
  type TauriListen,
  tauriTransport
} from "@/src/index";

/** An `invoke` that answers from a script and records what it was asked. */
class ScriptedInvoke {
  readonly calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  #answer: () => unknown = () => null;

  /** Answer every subsequent call by resolving this value. */
  resolveWith(value: unknown): void {
    this.#answer = () => value;
  }

  /** Answer every subsequent call by throwing this reason. */
  rejectWith(reason: unknown): void {
    this.#answer = () => {
      throw reason;
    };
  }

  readonly invoke: TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => {
    this.calls.push(args === undefined ? { command } : { command, args });
    return Promise.resolve(this.#answer() as T);
  };
}

/** A `listen` under the transport: records the channel and handlers, and
 * defers handing back the stop function until the test releases it — the
 * real one resolves after a webview round-trip, and the disposer must
 * behave in both orders. */
class ScriptedListen {
  readonly subscriptions: Array<{ channel: string; stopped: boolean }> = [];
  #handlers: Array<(message: { payload: unknown }) => void> = [];
  #resolvers: Array<() => void> = [];

  readonly listen: TauriListen = <T>(event: string, handler: (message: { payload: T }) => void) => {
    const subscription = { channel: event, stopped: false };
    this.subscriptions.push(subscription);
    this.#handlers.push(handler as (message: { payload: unknown }) => void);
    return new Promise<() => void>((resolve) => {
      this.#resolvers.push(() => {
        resolve(() => {
          subscription.stopped = true;
        });
      });
    });
  };

  /** Deliver one event payload to every subscription's handler. */
  fire(payload: unknown): void {
    for (const handler of this.#handlers) handler({ payload });
  }

  /** Resolve the pending subscriptions; their stop functions go live. */
  connect(): void {
    for (const release of this.#resolvers.splice(0)) release();
  }
}

/** A transport over a scripted invoke, fresh per test: each test's script
 * and call record are its own, and nothing leaks between tests. */
const tauriHarness = createFixture(() => {
  const invoke = new ScriptedInvoke();
  return { invoke, transport: createTauriTransport(invoke.invoke) };
});

/** Microtasks flushed: whatever a resolved subscription promise queued
 * inside the transport has run by the next macrotask. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

// ── Arbitraries: the request and response space, generated ──────────────────

const text = fc.string({ minLength: 0, maxLength: 24 });
const nullableText = fc.oneof(text, fc.constant(null));

const arbEntrySummary: fc.Arbitrary<EntrySummary> = fc.record({
  id: text,
  title: text,
  username: nullableText,
  url: nullableText,
  has_totp: fc.boolean(),
  has_passkey: fc.boolean()
});

const arbNewEntry: fc.Arbitrary<NewEntry> = fc.record({
  title: text,
  username: nullableText,
  password: nullableText,
  url: nullableText,
  otpauth: nullableText
});

const arbOtpImportCandidate = fc.record({
  issuer: nullableText,
  account: text,
  otpauth: nullableText,
  problem: nullableText
});

const arbOtpImportSelection = fc.record({
  title: text,
  username: nullableText,
  otpauth: text
});

/** The registries below are `satisfies`-typed the same way the protocol
 * package's round-trip suite types its own: adding an operation to the
 * Rust contract makes these objects fail to compile until the new
 * variant gets an arbitrary — the passthrough property then covers every
 * operation's shape, not a lucky sample. */
type MethodArbitraries = {
  [Method in RpcMethod["method"]]: fc.Arbitrary<Extract<RpcMethod, { method: Method }>>;
};

const methodArbitraries = {
  ping: fc.record({ method: fc.constant("ping") }),
  lock_database: fc.record({ method: fc.constant("lock_database") }),
  get_entries: fc.record({ method: fc.constant("get_entries"), origin: text }),
  get_totp: fc.record({ method: fc.constant("get_totp"), entry_id: text }),
  generate_passphrase: fc.record({
    method: fc.constant("generate_passphrase"),
    words: fc.nat(64),
    separator: text
  }),
  preview_otp_import: fc.record({
    method: fc.constant("preview_otp_import"),
    payload: text
  }),
  import_otp_accounts: fc.record({
    method: fc.constant("import_otp_accounts"),
    accounts: fc.array(arbOtpImportSelection)
  }),
  save_entry: fc.record({ method: fc.constant("save_entry"), entry: arbNewEntry })
} satisfies MethodArbitraries;

// RpcRequest is exactly `{ id, ...method }` because Rust uses serde
// flatten — the same derivation the protocol package's round-trip suite
// performs, preserving that flat wire contract in every generated case.
const arbRpcMethod: fc.Arbitrary<RpcMethod> = fc.oneof(
  ...(Object.values(methodArbitraries) as fc.Arbitrary<RpcMethod>[])
);

const arbRpcRequest: fc.Arbitrary<RpcRequest> = fc
  .tuple(fc.nat(), arbRpcMethod)
  .map(([id, method]) => ({ id, ...method }));

type ResultArbitraries = {
  [Type in RpcResult["type"]]: fc.Arbitrary<Extract<RpcResult, { type: Type }>>;
};

const resultArbitraries = {
  ping: fc.record({ type: fc.constant("ping") }),
  lock_database: fc.record({ type: fc.constant("lock_database") }),
  get_entries: fc.record({ type: fc.constant("get_entries"), entries: fc.array(arbEntrySummary) }),
  get_totp: fc.record({ type: fc.constant("get_totp"), code: text, seconds_remaining: fc.nat() }),
  generate_passphrase: fc.record({
    type: fc.constant("generate_passphrase"),
    value: text
  }),
  preview_otp_import: fc.record({
    type: fc.constant("preview_otp_import"),
    accounts: fc.array(arbOtpImportCandidate)
  }),
  import_otp_accounts: fc.record({
    type: fc.constant("import_otp_accounts"),
    imported: fc.nat()
  }),
  save_entry: fc.record({ type: fc.constant("save_entry"), id: text })
} satisfies ResultArbitraries;

const arbRpcError: fc.Arbitrary<RpcError> = fc.record({
  code: fc.constantFrom(
    "vault_locked",
    "bad_credentials",
    "vault_unreadable",
    "no_such_entry",
    "not_implemented",
    "bad_request"
  ),
  message: text
});

const arbRpcResponse: fc.Arbitrary<RpcResponse> = fc.record({
  id: fc.nat(),
  result: fc.oneof(
    ...(Object.values(resultArbitraries) as fc.Arbitrary<RpcResult>[]),
    fc.constant(null)
  ),
  error: fc.oneof(arbRpcError, fc.constant(null))
});

// ── The contract ─────────────────────────────────────────────────────────────

describe("createTauriTransport", () => {
  test("sends the complete request through the single rpc command", async () => {
    const { invoke, transport } = tauriHarness();
    invoke.resolveWith({ id: 7, result: { type: "ping" }, error: null });

    const response = await transport.request({ id: 7, method: "ping" });

    expect(invoke.calls).toEqual([
      { command: "rpc", args: { request: { id: 7, method: "ping" } } }
    ]);
    expect(response).toEqual({
      id: 7,
      result: { type: "ping" },
      error: null
    } satisfies RpcResponse);
  });

  test("normalizes invoke rejection as a disconnected transport error", async () => {
    const { invoke, transport } = tauriHarness();
    invoke.rejectWith(new Error("webview closed"));

    await expect(transport.request({ id: 1, method: "ping" })).rejects.toMatchObject({
      code: "disconnected"
    });
  });

  test("any request crosses as one rpc call, any response comes back untouched", async () => {
    await fc.assert(
      fc.asyncProperty(arbRpcRequest, arbRpcResponse, async (request, response) => {
        // Fresh wiring per run: a fixture's per-test lifecycle cannot
        // reach inside an `fc.assert` run, and each run needs its own
        // call record anyway.
        const invoke = new ScriptedInvoke();
        const transport = createTauriTransport(invoke.invoke);
        invoke.resolveWith(response);

        // Identity, not equality: the adapter must hand back the very
        // object invoke resolved with — no cloning, no reshaping.
        await expect(transport.request(request)).resolves.toBe(response);
        expect(invoke.calls).toEqual([{ command: "rpc", args: { request } }]);
      })
    );
  });

  test("any invoke rejection keeps its cause inside the disconnected error", async () => {
    await fc.assert(
      fc.asyncProperty(fc.string({ maxLength: 64 }), fc.boolean(), async (reason, asError) => {
        const invoke = new ScriptedInvoke();
        const transport = createTauriTransport(invoke.invoke);
        invoke.rejectWith(asError ? new Error(reason) : reason);

        const failing = transport.request({ id: 1, method: "ping" });

        await expect(failing).rejects.toMatchObject({ code: "disconnected" });
        await failing.catch((error: Error) => {
          expect(error.message).toContain(reason);
        });
      })
    );
  });
});

// ── The event pipe: session events in, callbacks out ─────────────────────────

describe("tauriTransport", () => {
  test("returns a disconnected fallback outside a Tauri webview", async () => {
    const transport = tauriTransport();

    expect(transport.connected).toBe(false);
    await expect(transport.request({} as RpcRequest)).rejects.toMatchObject({
      code: "disconnected",
      message:
        "not connected to the Castellan app: Tauri runtime is not available in this browser preview"
    });
  });
});

describe("onEvent", () => {
  test("subscribes to the castellan channel and forwards its payloads", () => {
    const invoke = new ScriptedInvoke();
    const scripted = new ScriptedListen();
    const transport = createTauriTransport(invoke.invoke, scripted.listen);

    const seen: Event[] = [];
    transport.onEvent((event) => seen.push(event));

    expect(scripted.subscriptions.map((s) => s.channel)).toEqual(["castellan://event"]);
    scripted.fire({ type: "database_unlocked" });
    scripted.fire({ type: "database_locked" });
    expect(seen).toEqual([{ type: "database_unlocked" }, { type: "database_locked" }]);
  });

  test("a disposer after the subscription resolves stops delivery and unsubscribes", async () => {
    const invoke = new ScriptedInvoke();
    const scripted = new ScriptedListen();
    const transport = createTauriTransport(invoke.invoke, scripted.listen);

    const seen: Event[] = [];
    const dispose = transport.onEvent((event) => seen.push(event));
    scripted.connect();
    await settle();

    dispose();
    scripted.fire({ type: "entry_changed", id: "entry-1" });

    expect(seen).toEqual([]);
    expect(scripted.subscriptions[0]?.stopped).toBe(true);
  });

  test("a disposer before the subscription resolves unsubscribes on arrival", async () => {
    const invoke = new ScriptedInvoke();
    const scripted = new ScriptedListen();
    const transport = createTauriTransport(invoke.invoke, scripted.listen);

    const seen: Event[] = [];
    const dispose = transport.onEvent((event) => seen.push(event));

    // The webview round-trip is still in flight when the face goes away.
    dispose();
    scripted.fire({ type: "database_locked" });
    scripted.connect();
    await settle();

    expect(seen).toEqual([]);
    expect(scripted.subscriptions[0]?.stopped).toBe(true);
  });

  test("any protocol event crosses the channel as the same object", async () => {
    // Property, not example: every event the host can push — lock state,
    // entry churn — must reach the face untouched, the same identity the
    // shell emitted.
    const arbEvent: fc.Arbitrary<Event> = fc.oneof(
      fc.constant({ type: "database_locked" } as Event),
      fc.constant({ type: "database_unlocked" } as Event),
      fc.record({ type: fc.constant("entry_changed"), id: text }) as fc.Arbitrary<Event>
    );

    await fc.assert(
      fc.asyncProperty(arbEvent, async (event) => {
        const invoke = new ScriptedInvoke();
        const scripted = new ScriptedListen();
        const transport = createTauriTransport(invoke.invoke, scripted.listen);

        const seen: Event[] = [];
        transport.onEvent((e) => seen.push(e));
        scripted.fire(event);

        expect(seen).toEqual([event]);
      })
    );
  });
});
