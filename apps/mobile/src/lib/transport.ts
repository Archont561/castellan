/**
 * The desktop face of the Transport seam.
 *
 * Tauri's `invoke` against the app's single `rpc` command: the frontend
 * speaks the same protocol as the browser extension, with the serialization
 * done by serde on the Rust side and by JSON-RPC-shaped types on this side.
 * Events arrive through Tauri's event system once the IPC server broadcasts
 * them; until then `onEvent` subscribes to nothing and says so in a comment
 * rather than pretending.
 */

import { DISCONNECTED, disconnected, type Transport } from "@castellan/core";
import type { Event, RpcRequest, RpcResponse } from "@castellan/protocol";
import { invoke } from "@tauri-apps/api/core";

export function tauriTransport(): Transport {
  return {
    get connected() {
      // Inside the webview the app is always there; a failed invoke rejects
      // and that rejection is the disconnection signal.
      return true;
    },

    async request(req: RpcRequest): Promise<RpcResponse> {
      try {
        // The command takes the whole envelope; the Rust side dispatches on
        // the tagged method, so adding a method is protocol work, never a
        // new Tauri command and never a new capability grant.
        return await invoke<RpcResponse>("rpc", { request: req });
      } catch (cause) {
        throw disconnected(String(cause));
      }
    },

    onEvent(cb: (event: Event) => void): () => void {
      // TODO(ipc): listen("castellan://event", ...) once the IPC server
      // forwards vault events into the Tauri event bus. The shape is already
      // fixed by the protocol crate; this is plumbing, not design.
      void cb;
      return () => {};
    }
  };
}

export { DISCONNECTED };
