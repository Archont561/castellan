/**
 * The client's invariants, proven the way the Rust side proves its crates:
 * properties over arbitrary inputs (fast-check here, proptest there), not
 * just examples. The Rust↔TypeScript symmetry is deliberate — the same
 * class of bug (a reused request id, a swallowed error code, a payload
 * field that stops traveling) should be catchable on whichever side it
 * regresses.
 *
 * Every property runs against the same scripted transport: no app, no
 * browser, no Tauri — the package's whole reason to exist is that these
 * guarantees hold for every face at once.
 */

import { describe, expect, test } from "bun:test";
import type { RpcRequest, RpcResponse } from "@castellan/protocol";
import fc from "fast-check";

import { CastellanClient, CastellanError } from "@/src/client";
import type { Transport } from "@/src/transport";

/** A transport that answers from a script and records what it was asked —
 * the property-testing twin of the scripted transport in client.test.ts,
 * pared down to what the properties need. */
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

const ok = (req: RpcRequest): RpcResponse => ({ id: req.id, result: { type: "ok" }, error: null });

describe("CastellanClient (properties)", () => {
  test("assigns every request a distinct, dense id", async () => {
    // The ids are the pairing key a multiplexing transport relies on
    // (native messaging shares one stream between every caller), so the
    // property is stronger than "different": ids must be exactly
    // 1..N with no hole and no reuse, however the calls interleave.
    // `asyncProperty` because the calls are awaited; 50 runs keep the
    // up-to-50-request bursts from making the suite slow.
    await fc.assert(
      fc.asyncProperty(fc.nat(49), async (spread) => {
        const calls = spread + 1;
        const transport = new ScriptedTransport(ok);
        const client = new CastellanClient(transport);

        await Promise.all(Array.from({ length: calls }, () => client.ping()));

        const ids = transport.sent.map((req) => req.id).sort((a, b) => a - b);
        expect(ids).toEqual(Array.from({ length: calls }, (_, index) => index + 1));
      }),
      { numRuns: 50 }
    );
  });

  test("an error on the wire keeps its stable code and message", async () => {
    // The UI switches on `code`; whatever string the app sent is what the
    // face must receive, byte for byte, message included.
    await fc.assert(
      fc.asyncProperty(
        fc.record({
          code: fc.string({ minLength: 1, maxLength: 32 }),
          message: fc.string({ maxLength: 64 })
        }),
        async (failure) => {
          const transport = new ScriptedTransport((req) => ({
            id: req.id,
            result: null,
            error: failure
          }));
          const client = new CastellanClient(transport);

          const failing = client.getTotp("some-entry");

          await expect(failing).rejects.toBeInstanceOf(CastellanError);
          await expect(failing).rejects.toMatchObject(failure);
        }
      )
    );
  });

  test("spreads the method flat onto the request, whatever the origin", () => {
    // The wire shape is `{ id, ...method }` — serde's `flatten` on the Rust
    // side. The property pins it for arbitrary origins: the origin a page
    // hands the extension arrives verbatim, not escaped, trimmed or
    // re-encoded on the way into the request.
    fc.assert(
      fc.property(fc.string({ maxLength: 64 }), (origin) => {
        const transport = new ScriptedTransport(ok);
        const client = new CastellanClient(transport);

        void client.getEntries(origin);

        expect(transport.sent[0]).toEqual({ id: 1, method: "get_entries", origin });
      })
    );
  });

  test("id sequences are per client — a second client starts over", async () => {
    // A transport multiplexing several clients (the extension's port is
    // one such) cannot assume ids are globally unique: each client owns
    // its own counter. This property pins that contract so a future
    // "clever" shared counter cannot sneak in silently.
    await fc.assert(
      fc.asyncProperty(fc.nat(20), async (firstBurst) => {
        const transport = new ScriptedTransport(ok);
        const first = new CastellanClient(transport);
        const second = new CastellanClient(transport);

        await Promise.all(Array.from({ length: firstBurst }, () => first.ping()));
        await second.ping();

        expect(transport.sent.at(-1)?.id).toBe(1);
        expect(second === first).toBe(false);
      })
    );
  });
});
