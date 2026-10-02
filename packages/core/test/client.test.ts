/**
 * The shared client's contract, proven against a scripted transport. Face
 * methods are generated elsewhere; this suite pins the mechanics every one
 * of those generated clients inherits.
 */
import { describe, expect, test } from "bun:test";

import type { Event, RpcRequest, RpcResponse } from "@castellan/protocol";
import { CastellanError, RpcClient } from "@/src/client";
import { disconnected, type Transport } from "@/src/transport";

/** The smallest generated-client analogue needed to exercise the base. */
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

/** A transport that answers from a script and records what it was asked. */
class ScriptedTransport implements Transport {
  readonly sent: RpcRequest[] = [];
  #handler: (req: RpcRequest) => RpcResponse;
  #listeners: ((event: Event) => void)[] = [];

  constructor(handler: (req: RpcRequest) => RpcResponse) {
    this.#handler = handler;
  }

  get connected(): boolean {
    return true;
  }

  request(req: RpcRequest): Promise<RpcResponse> {
    this.sent.push(req);
    return Promise.resolve(this.#handler(req));
  }

  onEvent(cb: (event: Event) => void): () => void {
    this.#listeners.push(cb);
    return () => {
      this.#listeners = this.#listeners.filter((listener) => listener !== cb);
    };
  }

  emit(event: Event): void {
    for (const listener of this.#listeners) listener(event);
  }
}

describe("RpcClient", () => {
  test("builds the flat wire shape the Rust dispatcher deserializes", () => {
    const transport = new ScriptedTransport((req) => ({
      id: req.id,
      result: { type: "get_entries", entries: [] },
      error: null
    }));
    const client = new TestClient(transport);

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
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: { type: "get_totp", code: "287082", seconds_remaining: 7 },
      error: null
    }));
    const client = new TestClient(transport);

    const totp = await client.getTotp("some-entry-id");

    expect(totp).toEqual({ code: "287082", secondsRemaining: 7 });
  });

  test("rejects a success variant belonging to another operation", async () => {
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: { type: "ping" },
      error: null
    }));
    const client = new TestClient(transport);

    await expect(client.getEntries("https://github.com")).rejects.toMatchObject({
      code: "unexpected_result"
    });
  });

  test("raises CastellanError with the app's stable code", async () => {
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: null,
      error: { code: "vault_locked", message: "the vault is locked" }
    }));
    const client = new TestClient(transport);

    const failing = client.getTotp("some-entry-id");

    await expect(failing).rejects.toBeInstanceOf(CastellanError);
    await failing.catch((error: CastellanError) => {
      expect(error.code).toBe("vault_locked");
    });
  });

  test("delivers pushed events to subscribers, and unsubscribes", () => {
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: { type: "ping" },
      error: null
    }));
    const client = new TestClient(transport);

    const seen: Event[] = [];
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
