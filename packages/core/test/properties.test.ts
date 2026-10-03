/**
 * The shared client's invariants, proven over arbitrary inputs. Generated
 * face methods are thin calls into this base, so request ids, errors and
 * flat payloads need one property suite rather than one suite per app.
 *
 * Each property run builds its own client over its own scripted transport
 * (from ./support.ts, where the pair is defined once) — a fixture's
 * per-test lifecycle cannot reach inside an `fc.assert` run, and each run
 * needs independent state anyway.
 */

import { describe, expect, test } from "bun:test";
import fc from "fast-check";

import { CastellanError } from "@/src/client";

import { ScriptedTransport, success, TestClient } from "./support";

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
