/**
 * Transport-independent mechanics shared by every generated face client.
 *
 * The protocol macro decides which operations each face can call, and xtask
 * emits the public methods into that app. This base owns only invariants that
 * must never vary by face: request ids, flat envelopes, result correlation,
 * stable errors and pushed events.
 */

import {
  PROTOCOL_VERSION,
  type RpcError,
  type RpcErrorCode,
  type RpcMethod,
  type RpcRequest,
  type RpcResponse,
  type RpcResult
} from "@castellan/protocol";

import { DISCONNECTED, type Transport } from "./transport";

/** A method tag from the generated protocol union. */
export type RpcMethodName = RpcMethod["method"];

/** The request payload for `M`, without the method discriminator. */
export type RpcParams<M extends RpcMethodName> = Omit<Extract<RpcMethod, { method: M }>, "method">;

/** The success variant correlated with method `M`. */
export type RpcResultFor<M extends RpcMethodName> = Extract<RpcResult, { type: M }>;

/** Failures produced locally rather than carried on the Rust wire. */
export type ClientErrorCode = typeof DISCONNECTED | "malformed_response" | "unexpected_result";

/** Every stable error code a face can receive. */
export type CastellanErrorCode = RpcErrorCode | ClientErrorCode;

/** What went wrong on a call, with the stable code the UI can switch on. */
export class CastellanError extends Error {
  constructor(
    readonly code: CastellanErrorCode,
    message: string
  ) {
    super(message);
    this.name = "CastellanError";
  }
}

/** Shared base for the desktop, mobile and web-extension generated clients. */
export class RpcClient {
  private nextId = 1;
  private readonly transport: Transport;

  constructor(transport: Transport) {
    this.transport = transport;
  }

  /** The protocol version this client speaks, for handshakes. */
  get protocolVersion(): number {
    return PROTOCOL_VERSION;
  }

  /** Subscribe to pushed events (lock state changes, entry changes). */
  onEvent(cb: Parameters<Transport["onEvent"]>[0]): () => void {
    return this.transport.onEvent(cb);
  }

  /**
   * Turn a generated operation into a request/response pair.
   *
   * `expectedType` is generated from the same macro entry as `method`, so a
   * server returning another operation's success payload is rejected here
   * rather than becoming an empty value in a UI.
   */
  protected async call<M extends RpcMethodName>(
    method: Extract<RpcMethod, { method: M }>,
    expectedType: M
  ): Promise<RpcResultFor<M>> {
    const request: RpcRequest = { id: this.nextId++, ...method };
    const response: RpcResponse = await this.transport.request(request);

    if (response.error) {
      throw toError(response.error);
    }
    if (!response.result) {
      throw new CastellanError("malformed_response", `response ${response.id} had no result`);
    }
    if (response.result.type !== expectedType) {
      throw new CastellanError(
        "unexpected_result",
        `expected a ${expectedType} result, got ${response.result.type}`
      );
    }
    return response.result as RpcResultFor<M>;
  }
}

function toError(error: RpcError): CastellanError {
  return new CastellanError(error.code, error.message);
}

export { DISCONNECTED };
