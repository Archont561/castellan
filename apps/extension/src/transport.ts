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
import type {
  ClientMessage,
  Event,
  HostMessage,
  RpcRequest,
  RpcResponse
} from "@castellan/protocol";

/** The native host name; must match the manifest the app installs. */
export const HOST_NAME = "app.castellan.host";

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
    port?.onDisconnect.addListener(() => {});
    const fresh = getRuntime().connectNative(HOST_NAME);

    fresh.onMessage.addListener((message: HostMessage) => {
      if (message.kind === "response" && message.response) {
        const waiter = pending.get(message.response.id);
        if (waiter) {
          pending.delete(message.response.id);
          waiter.resolve(message.response);
        }
        return;
      }
      if (message.kind === "event" && message.event) {
        for (const listener of listeners) listener(message.event);
      }
      // kind === "hello" is the app's handshake; the client library sends
      // its own hello on first use, so nothing to do here yet.
    });

    fresh.onDisconnect.addListener(() => {
      alive = false;
      // Every waiting caller learns the app went away now, not on a timer.
      for (const waiter of pending.values()) {
        waiter.reject(disconnected("the native host disconnected"));
      }
      pending.clear();
    });

    port = fresh;
    alive = true;
    return fresh;
  }

  return {
    get connected() {
      return alive;
    },

    request(req: RpcRequest): Promise<RpcResponse> {
      return new Promise<RpcResponse>((resolve, reject) => {
        const target = alive ? (port ?? connect()) : connect();
        pending.set(req.id, { resolve, reject });

        // The hello rides ahead of the first request on a fresh connection:
        // the app logs the client's protocol version, and a future
        // capability negotiation has its hook without a protocol change.
        try {
          target.postMessage({ kind: "hello", protocol_version: 1 } satisfies ClientMessage);
          target.postMessage({ kind: "request", request: req } satisfies ClientMessage);
        } catch (cause) {
          pending.delete(req.id);
          reject(disconnected(String(cause)));
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
