/**
 * The Transport seam: how a request physically travels, without any opinion
 * about what the request means.
 *
 * Two adapters exist today:
 *
 * - `@castellan/tauri` — Tauri's `invoke`, one `rpc` command shared by the
 *   desktop and mobile faces;
 * - `apps/extension/src/transport.ts` — the browser's native-messaging port,
 *   framed exactly like the IPC socket and multiplexed by request id.
 *
 * Adapters stay outside this package so core never drags Tauri or browser
 * globals into consumers that do not use them. The client mechanics — the
 * part that has to be identical everywhere — live here.
 */

import type { Event, RpcRequest, RpcResponse } from "@castellan/protocol";

/**
 * A connection to the Castellan app, whatever it is made of.
 *
 * Implementations must:
 *
 * - pair responses to requests by `id` (native messaging is a single stream
 *   shared by every caller);
 * - deliver pushed events to every subscriber;
 * - surface the app going away as a rejected request with a `disconnected`
 *   error, never as a hang — and be re-connectable after that, because a
 *   browser restarts its service worker whenever it feels like it and the
 *   reconnect must be invisible.
 */
export interface Transport {
  /** Send one request and resolve with its response. */
  request(req: RpcRequest): Promise<RpcResponse>;
  /** Subscribe to pushed events. Returns an unsubscribe function. */
  onEvent(cb: (event: Event) => void): () => void;
  /** Whether the underlying connection is currently alive. */
  readonly connected: boolean;
}

/** The error code every transport uses for "the app is not there". */
export const DISCONNECTED = "disconnected";

/** Build the transport-independent shape of "the app went away". */
export function disconnected(cause?: string): Error & { code: typeof DISCONNECTED } {
  const error = new Error(
    cause ? `not connected to the Castellan app: ${cause}` : "not connected to the Castellan app"
  ) as Error & { code: typeof DISCONNECTED };
  error.code = DISCONNECTED;
  return error;
}
