/**
 * Tauri's adapter to the transport-independent RPC client.
 *
 * Desktop and mobile deliberately share this package: they both invoke the
 * same single Rust command, and future event-bus plumbing must not drift
 * between faces. The invoke function remains injectable so this boundary can
 * be tested without a webview.
 */

import { disconnected, type Transport } from "@castellan/core";
import type { Event, RpcRequest, RpcResponse } from "@castellan/protocol";
import { invoke } from "@tauri-apps/api/core";

/** The generic portion of Tauri's `invoke`, kept small for test doubles. */
export type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/** Build a Tauri transport around an invoke implementation. */
export function createTauriTransport(call: TauriInvoke): Transport {
  return {
    get connected() {
      // Inside the webview the app is always there; a failed invoke rejects
      // and that rejection is the disconnection signal.
      return true;
    },

    async request(req: RpcRequest): Promise<RpcResponse> {
      try {
        // The command takes the whole envelope; Rust dispatches on the tagged
        // method, so adding an operation never creates another Tauri command.
        return await call<RpcResponse>("rpc", { request: req });
      } catch (cause) {
        throw disconnected(String(cause));
      }
    },

    onEvent(cb: (event: Event) => void): () => void {
      // TODO(ipc): listen("castellan://event", ...) once the IPC server
      // forwards vault events into the Tauri event bus.
      void cb;
      return () => {};
    }
  };
}

/** The production transport used by both native faces. */
export function tauriTransport(): Transport {
  return createTauriTransport(invoke);
}
