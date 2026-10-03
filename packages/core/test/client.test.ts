/**
 * The shared client's contract, proven against a scripted transport. Face
 * methods are generated elsewhere; this suite pins the mechanics every one
 * of those generated clients inherits. The wiring is the shared fixture in
 * ./support.ts — the same pair the property suite drives, built once.
 */
import { describe, expect, test } from "bun:test";

import { CastellanError } from "@/src/client";
import { disconnected } from "@/src/transport";

import { scriptedClient } from "./support";

describe("RpcClient", () => {
  test("builds the flat wire shape the Rust dispatcher deserializes", () => {
    const { client, transport } = scriptedClient();

    void client.getEntries("https://github.com");

    const request = transport.sent[0];
    if (request === undefined) {
      throw new Error("no request was sent");
    }
    expect(request).toEqual({
      id: 1,
      method: "get_entries",
      origin: "https://github.com"
    });
  });

  test("narrows correlated results for generated methods", async () => {
    const { client, transport } = scriptedClient();
    transport.answerWith(() => ({
      id: 1,
      result: { type: "get_totp", code: "287082", seconds_remaining: 7 },
      error: null
    }));

    const totp = await client.getTotp("some-entry-id");

    expect(totp).toEqual({ code: "287082", secondsRemaining: 7 });
  });

  test("rejects a success variant belonging to another operation", async () => {
    const { client, transport } = scriptedClient();
    transport.answerWith(() => ({
      id: 1,
      result: { type: "ping" },
      error: null
    }));

    await expect(client.getEntries("https://github.com")).rejects.toMatchObject({
      code: "unexpected_result"
    });
  });

  test("raises CastellanError with the app's stable code", async () => {
    const { client, transport } = scriptedClient();
    transport.answerWith(() => ({
      id: 1,
      result: null,
      error: { code: "vault_locked", message: "the vault is locked" }
    }));

    const failing = client.getTotp("some-entry-id");

    await expect(failing).rejects.toBeInstanceOf(CastellanError);
    await failing.catch((error: CastellanError) => {
      expect(error.code).toBe("vault_locked");
    });
  });

  test("delivers pushed events to subscribers, and unsubscribes", () => {
    const { client, transport } = scriptedClient();

    const seen: Parameters<Parameters<typeof client.onEvent>[0]>[0][] = [];
    const stop = client.onEvent((event) => seen.push(event));
    transport.emit({ type: "database_locked" });
    stop();
    transport.emit({ type: "database_unlocked" });

    expect(seen).toEqual([{ type: "database_locked" }]);
  });

  test("disconnected() carries the DISCONNECTED code", () => {
    const error = disconnected("port closed");
    expect(error.code).toBe("disconnected");
    expect(error.message).toContain("port closed");
  });
});
