/**
 * The shared client's invariants, proven over arbitrary inputs. Generated
 * face methods are thin calls into this base, so request ids, errors and flat
 * payloads need one property suite rather than one suite per app.
 */

import { describe, expect, test } from "bun:test";
import type { RpcRequest, RpcResponse } from "@castellan/protocol";
import fc from "fast-check";

import { CastellanError, RpcClient } from "@/src/client";
import type { Transport } from "@/src/transport";

class TestClient extends RpcClient {
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

class ScriptedTransport implements Transport {
  readonly sent: RpcRequest[] = [];
  readonly #answer: (req: RpcRequest) => RpcResponse;

  constructor(answer: (req: RpcRequest) => RpcResponse) {
    this.#answer = answer;
  }

  get connected(): boolean {
    return true;
  }

  onEvent(): () => void {
    return () => {};
  }

  request(req: RpcRequest): Promise<RpcResponse> {
    this.sent.push(req);
    return Promise.resolve(this.#answer(req));
  }
}

const success = (req: RpcRequest): RpcResponse => {
  if (req.method === "get_entries") {
    return { id: req.id, result: { type: "get_entries", entries: [] }, error: null };
  }
  return { id: req.id, result: { type: "ping" }, error: null };
};

describe("RpcClient (properties)", () => {
  test("assigns every request a distinct, dense id", async () => {
    await fc.assert(
      fc.asyncProperty(fc.nat(49), async (spread) => {
        const calls = spread + 1;
        const transport = new ScriptedTransport(success);
        const client = new TestClient(transport);

        await Promise.all(Array.from({ length: calls }, () => client.ping()));

        const ids = transport.sent.map((req) => req.id).sort((a, b) => a - b);
        expect(ids).toEqual(Array.from({ length: calls }, (_, index) => index + 1));
      }),
      { numRuns: 50 }
    );
  });

  test("an error on the wire keeps its stable code and message", async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.record({
          code: fc.constantFrom("vault_locked", "no_such_entry", "not_implemented"),
          message: fc.string({ maxLength: 64 })
        }),
        async (failure) => {
          const transport = new ScriptedTransport((req) => ({
            id: req.id,
            result: null,
            error: failure
          }));
          const client = new TestClient(transport);

          const failing = client.getTotp("some-entry");

          await expect(failing).rejects.toBeInstanceOf(CastellanError);
          await expect(failing).rejects.toMatchObject(failure);
        }
      )
    );
  });

  test("spreads the method flat onto the request, whatever the origin", async () => {
    await fc.assert(
      fc.asyncProperty(fc.string({ maxLength: 64 }), async (origin) => {
        const transport = new ScriptedTransport(success);
        const client = new TestClient(transport);

        await client.getEntries(origin);

        expect(transport.sent[0]).toEqual({ id: 1, method: "get_entries", origin });
      })
    );
  });

  test("id sequences are per client—a second client starts over", async () => {
    await fc.assert(
      fc.asyncProperty(fc.nat(20), async (firstBurst) => {
        const transport = new ScriptedTransport(success);
        const first = new TestClient(transport);
        const second = new TestClient(transport);

        await Promise.all(Array.from({ length: firstBurst }, () => first.ping()));
        await second.ping();

        expect(transport.sent.at(-1)?.id).toBe(1);
        expect(second === first).toBe(false);
      })
    );
  });
});
