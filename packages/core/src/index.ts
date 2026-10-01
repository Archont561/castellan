/**
 * @castellan/core — the transport-agnostic client.
 *
 * Every face of Castellan that talks to the app does it through this
 * package: the request building, the id multiplexing, the error narrowing.
 * Only the Transport implementation is face-specific, and it lives in the
 * app that needs it.
 */
export { CastellanClient, CastellanError, DISCONNECTED } from "./client";
export type { Transport } from "./transport";
export { disconnected } from "./transport";
