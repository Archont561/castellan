/**
 * The one wiring every client suite needs, defined once: a client over a
 * scripted transport. Before this module, client.test.ts and
 * properties.test.ts each carried their own copy of the pair — and they
 * had already drifted (one transport supported pushed events, the other
 * silently dropped them), which is the exact failure mode the repo's
 * fixtures convention exists to prevent.
 *
 * `scriptedClient` is the per-test fixture for the example-driven suite.
 * The property suite constructs the same pair inside each `fc.assert`
 * run, where a fixture's per-test lifecycle cannot reach — it imports
 * the classes from here so the pair still exists exactly once.
 */
import type { Event, RpcRequest, RpcResponse } from "@castellan/protocol";
import { createFixture } from "@castellan/utils/fixtures";

import { RpcClient } from "@/src/client";
import type { Transport } from "@/src/transport";

/** The smallest generated-client analogue needed to exercise the base. */
export class TestClient extends RpcClient {
  async ping(): Promise<void> {
    await this.call({ method: "ping" }, "ping");
  }

  async getEntries(origin: string): Promise<unknown[]> {
    const result = await this.call({ method: "get_entries", origin }, "get_entries");
    return result.entries;
  }

  async getTotp(entryId: string): Promise<{ code: string; secondsRemaining: number }> {
    const result = await this.call({ method: "get_totp", entry_id: entryId }, "get_totp");
    return { code: result.code, secondsRemaining: result.seconds_remaining };
  }
}

/** The default script: answer the operation the request asked for. */
export function success(req: RpcRequest): RpcResponse {
  if (req.method === "get_entries") {
    return { id: req.id, result: { type: "get_entries", entries: [] }, error: null };
  }
  return { id: req.id, result: { type: "ping" }, error: null };
}

/**
 * A transport that answers from a script and records what it was asked.
 * Re-script in place with `answerWith`; push events at the client with
 * `emit` — the two halves a suite needs to drive the base class.
 */
export class ScriptedTransport implements Transport {
  readonly sent: RpcRequest[] = [];
  #answer: (req: RpcRequest) => RpcResponse = success;
  #listeners: ((event: Event) => void)[] = [];

  constructor(answer: (req: RpcRequest) => RpcResponse = success) {
    this.#answer = answer;
  }

  /** Swap the script — a fresh answer without rebuilding the wiring. */
  answerWith(answer: (req: RpcRequest) => RpcResponse): void {
    this.#answer = answer;
  }

  get connected(): boolean {
    return true;
  }

  request(req: RpcRequest): Promise<RpcResponse> {
    this.sent.push(req);
    return Promise.resolve(this.#answer(req));
  }

  onEvent(cb: (event: Event) => void): () => void {
    this.#listeners.push(cb);
    return () => {
      this.#listeners = this.#listeners.filter((listener) => listener !== cb);
    };
  }

  /** Deliver a pushed event the way a real transport would. */
  emit(event: Event): void {
    for (const listener of this.#listeners) listener(event);
  }
}

/**
 * A client over a scripted transport, fresh per test: each test's script
 * is its own (`transport.answerWith(...)`), and no `sent` record or event
 * listener leaks from one test into the next.
 */
export const scriptedClient = createFixture(() => {
  const transport = new ScriptedTransport();
  const client = new TestClient(transport);
  return { client, transport };
});
