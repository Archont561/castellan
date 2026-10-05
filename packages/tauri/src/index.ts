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
import { listen } from "@tauri-apps/api/event";

/** The generic portion of Tauri's `invoke`, kept small for test doubles. */
export type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

/**
 * The generic portion of Tauri's `listen`, kept small for test doubles:
 * subscribe to a channel, receive a stop function (possibly later — the
 * real one resolves once the webview round-trip completes).
 */
export type TauriListen = <T>(
  event: string,
  handler: (message: { payload: T }) => void
) => Promise<() => void>;

/**
 * The channel the Rust shells forward session events onto — unlock, lock,
 * entry changes. The shells' `EVENT_CHANNEL` and this constant are the
 * same name on purpose; changing one without the other orphans the faces.
 */
const EVENT_CHANNEL = "castellan://event";

/** Build a Tauri transport around invoke and listen implementations. */
export function createTauriTransport(
  call: TauriInvoke,
  subscribe: TauriListen = listen
): Transport {
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
      // The Rust shell forwards session events onto the webview event bus
      // on this channel; the subscription races nothing but its own
      // teardown, so a dispose before the stop function arrives simply
      // stops the one event still in flight.
      let active = true;
      let stop: (() => void) | null = null;
      void subscribe<Event>(EVENT_CHANNEL, ({ payload }) => {
        if (active) cb(payload);
      }).then((unlisten) => {
        if (active) {
          stop = unlisten;
        } else {
          unlisten();
        }
      });
      return () => {
        active = false;
        stop?.();
      };
    }
  };
}

/** Whether this page is running inside a Tauri webview rather than a browser preview. */
function hasTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in globalThis;
}

/**
 * The browser preview has no native bridge. Keep it renderable and surface the
 * same transport-level error as a disconnected app instead of leaking the
 * Tauri API stub's `invoke` TypeError into the UI.
 */
function unavailableTauriTransport(): Transport {
  return {
    connected: false,
    request: async () => {
      throw disconnected("Tauri runtime is not available in this browser preview");
    },
    onEvent: () => () => {}
  };
}

/** The production transport used by both native faces. */
export function tauriTransport(): Transport {
  return hasTauriRuntime() ? createTauriTransport(invoke) : unavailableTauriTransport();
}
