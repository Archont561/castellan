/**
 * The extension face of the Transport seam.
 *
 * browser.runtime.connectNative + the native-messaging wire format, which is
 * byte-for-byte the IPC framing in castellan-ipc: a 4-byte LE length prefix
 * and JSON. The host process (the app binary, `--native-host`) pumps frames
 * between this port and the app's socket without parsing them.
 *
 * Multiplexing by request id and reconnect-on-sleep live here, because they
 * are properties of *this* transport: an MV3 service worker is killed by
 * Chrome whenever it feels like it, which tears the port down. The client
 * code in @castellan/core never knows — it just sees a transport whose
 * requests eventually resolve.
 */

import { DISCONNECTED, disconnected, type Transport } from "@castellan/core";
import {
  type ClientMessage,
  type Event,
  type HostMessage,
  PROTOCOL_VERSION,
  type RpcRequest,
  type RpcResponse
} from "@castellan/protocol";
import { NATIVE_HOST_NAME } from "@/src/generated/native-host";

/** The generated native host name, exported for diagnostics and tests. */
export const HOST_NAME = NATIVE_HOST_NAME;

interface NativePort {
  postMessage(message: ClientMessage): void;
  onMessage: {
    addListener(listener: (message: HostMessage) => void): void;
    removeListener(listener: (message: HostMessage) => void): void;
  };
  onDisconnect: {
    addListener(listener: () => void): void;
    removeListener(listener: () => void): void;
  };
}

export interface NativeMessaging {
  connectNative(name: string): NativePort;
}

export function nativeMessagingTransport(getRuntime: () => NativeMessaging): Transport {
  const pending = new Map<
    number,
    { resolve: (r: RpcResponse) => void; reject: (e: Error) => void }
  >();
  const listeners = new Set<(event: Event) => void>();

  let port: NativePort | undefined;
  let alive = false;

  function connect(): NativePort {
    const fresh = getRuntime().connectNative(HOST_NAME);

    fresh.onMessage.addListener((message: HostMessage) => {
      if (message.kind === "response") {
        const waiter = pending.get(message.response.id);
        if (waiter) {
          pending.delete(message.response.id);
          waiter.resolve(message.response);
        }
        return;
      }
      if (message.kind === "event") {
        for (const listener of listeners) listener(message.event);
      }
      // kind === "hello" is the app's answer to the handshake sent below;
      // capability-aware UI will consume it when that state lands.
    });

    fresh.onDisconnect.addListener(() => {
      // A late disconnect from an old port must not tear down a replacement
      // connection or reject requests already sent over that replacement.
      if (port !== fresh) return;
      port = undefined;
      alive = false;
      // Every waiting caller learns the app went away now, not on a timer.
      for (const waiter of pending.values()) {
        waiter.reject(disconnected("the native host disconnected"));
      }
      pending.clear();
    });

    port = fresh;
    alive = true;
    // Exactly one hello per physical connection. Reconnect creates a new
    // port and therefore sends a fresh handshake before its first request.
    fresh.postMessage({
      kind: "hello",
      protocol_version: PROTOCOL_VERSION
    } satisfies ClientMessage);
    return fresh;
  }

  return {
    get connected() {
      return alive;
    },

    request(req: RpcRequest): Promise<RpcResponse> {
      return new Promise<RpcResponse>((resolve, reject) => {
        pending.set(req.id, { resolve, reject });
        try {
          const target = alive ? (port ?? connect()) : connect();
          target.postMessage({ kind: "request", request: req } satisfies ClientMessage);
        } catch (cause) {
          port = undefined;
          alive = false;
          const failure = disconnected(String(cause));
          for (const waiter of pending.values()) waiter.reject(failure);
          pending.clear();
        }
      });
    },

    onEvent(cb: (event: Event) => void): () => void {
      listeners.add(cb);
      return () => {
        listeners.delete(cb);
      };
    }
  };
}

export { DISCONNECTED };
