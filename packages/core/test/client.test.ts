/**
 * The client's contract, proven against a scripted transport. No app, no
 * browser, no Tauri: the same test file would pass in every face's CI, which
 * is the point of having the client in a package.
 */
import { describe, expect, test } from "bun:test";

import type { Event, RpcRequest, RpcResponse } from "@castellan/protocol";
import { CastellanClient, CastellanError } from "@/src/client";
import { disconnected, type Transport } from "@/src/transport";

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

describe("CastellanClient", () => {
  test("builds the wire shape the Rust dispatcher deserializes", () => {
    const transport = new ScriptedTransport((req) => ({
      id: req.id,
      result: { type: "entries", entries: [] },
      error: null
    }));
    const client = new CastellanClient(transport);

    void client.getEntries("https://github.com");

    const request = transport.sent[0];
    if (request === undefined) {
      throw new Error("no request was sent");
    }
    expect(request.id).toBe(1);
    // The internally-tagged method, flattened onto the request: id and the
    // method's payload fields share one object — exactly what serde sees on
    // the other side of the wire.
    expect(request).toEqual({
      id: 1,
      method: "get_entries",
      origin: "https://github.com"
    });
  });

  test("narrows totp results and renames seconds_remaining", async () => {
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: { type: "totp", code: "287082", seconds_remaining: 7 },
      error: null
    }));
    const client = new CastellanClient(transport);

    const totp = await client.getTotp("some-entry-id");

    expect(totp.code).toBe("287082");
    expect(totp.secondsRemaining).toBe(7);
  });

  test("raises CastellanError with the app's stable code", async () => {
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: null,
      error: { code: "vault_locked", message: "the vault is locked" }
    }));
    const client = new CastellanClient(transport);

    const failing = client.getTotp("some-entry-id");

    await expect(failing).rejects.toBeInstanceOf(CastellanError);
    await failing.catch((error: CastellanError) => {
      expect(error.code).toBe("vault_locked");
    });
  });

  test("delivers pushed events to subscribers, and unsubscribes", () => {
    const transport = new ScriptedTransport(() => ({
      id: 1,
      result: { type: "ok" },
      error: null
    }));
    const client = new CastellanClient(transport);

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
