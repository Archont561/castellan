/**
 * The Castellan client: typed methods over any Transport.
 *
 * This is the "same scoped logic" promise for the TypeScript side. The
 * desktop app, the mobile app, the extension and the tests all call
 * `client.getEntries(origin)` — one place builds the request, checks the
 * error code, and narrows the result type. A new method is added here once,
 * and every face gets it by upgrading the package, not by copying code.
 */

import {
  type EntrySummary,
  type NewEntry,
  PROTOCOL_VERSION,
  type RpcError,
  type RpcMethod,
  type RpcRequest,
  type RpcResponse,
  type RpcResult
} from "@castellan/protocol";

import { DISCONNECTED, type Transport } from "./transport";

/** What went wrong on a call, with the stable code the UI can switch on. */
export class CastellanError extends Error {
  constructor(
    readonly code: string,
    message: string
  ) {
    super(message);
    this.name = "CastellanError";
  }
}

/** The client. One instance per app lifetime; reconnects under it. */
export class CastellanClient {
  private nextId = 1;
  private readonly transport: Transport;

  constructor(transport: Transport) {
    this.transport = transport;
  }

  /** The protocol version this client speaks, for handshakes. */
  get protocolVersion(): number {
    return PROTOCOL_VERSION;
  }

  /** Liveness probe. */
  async ping(): Promise<void> {
    await this.call({ method: "ping" });
  }

  /** Entries relevant to an origin, for fill UIs and quick search. */
  async getEntries(origin: string): Promise<EntrySummary[]> {
    const result = await this.call({ method: "get_entries", origin });
    return result.type === "entries" ? result.entries : [];
  }

  /** The current TOTP code for an entry, and its remaining seconds. */
  async getTotp(entryId: string): Promise<{ code: string; secondsRemaining: number }> {
    const result = await this.call({ method: "get_totp", entry_id: entryId });
    if (result.type !== "totp") {
      throw new CastellanError("unexpected_result", `expected a totp result, got ${result.type}`);
    }
    return { code: result.code, secondsRemaining: result.seconds_remaining };
  }

  /** A generated passphrase. */
  async generatePassphrase(words: number, separator: string): Promise<string> {
    const result = await this.call({ method: "generate_passphrase", words, separator });
    return result.type === "passphrase" ? result.value : "";
  }

  /** Save an entry captured by a save prompt. Returns the new entry's id. */
  async saveEntry(entry: NewEntry): Promise<string> {
    const result = await this.call({ method: "save_entry", entry });
    return result.type === "saved" ? result.id : "";
  }

  /** Lock the vault. Never throws: a lock that fails is reported, not ignored. */
  async lock(): Promise<void> {
    await this.call({ method: "lock_database" });
  }

  /** Subscribe to pushed events (lock state changes, entry changes). */
  onEvent(cb: Parameters<Transport["onEvent"]>[0]): () => void {
    return this.transport.onEvent(cb);
  }

  /**
   * The one place a method call becomes a request/response pair. The method
   * variant is *spread* onto the request — serde's `flatten` on the Rust
   * side, `{ id, ...method }` here — so every payload field (origin,
   * entry_id, words, …) travels with the tag.
   */
  private async call(method: RpcMethod): Promise<RpcResult> {
    const request: RpcRequest = { id: this.nextId++, ...method };
    const response: RpcResponse = await this.transport.request(request);

    if (response.error) {
      throw toError(response.error);
    }
    if (!response.result) {
      // The protocol says exactly one of result/error is set; an answer with
      // neither is a bug on the app side, and a bug should not typecheck as
      // success.
      throw new CastellanError("malformed_response", `response ${response.id} had no result`);
    }
    return response.result;
  }
}

function toError(error: RpcError): CastellanError {
  return new CastellanError(error.code, error.message);
}

export { DISCONNECTED };
