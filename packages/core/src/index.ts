/**
 * @castellan/core — transport-independent client mechanics.
 *
 * Face-specific public methods are generated into the desktop, mobile and
 * extension apps. They all extend the same RpcClient so request construction,
 * response validation and error behavior remain one implementation.
 */

export type {
  CastellanErrorCode,
  ClientErrorCode,
  RpcMethodName,
  RpcParams,
  RpcResultFor
} from "./client";
export { CastellanError, DISCONNECTED, RpcClient } from "./client";
export type { Transport } from "./transport";
export { disconnected } from "./transport";
