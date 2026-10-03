/**
 * The extension face of the Transport seam.
 *
 * browser.runtime.connectNative + the native-messaging wire format, which is
 * byte-for-byte the IPC framing in castellan-ipc: a 4-byte LE length prefix
 * and JSON. The host process (the app binary, `--native-host`) pumps frames
 * between this port and the app's socket without parsing them.
 *
 * Protocol v3 puts an association handshake in front of the traffic: the
 * hello carries the extension's face, its version, and a claim on its
 * stored key; the app answers with a nonce challenge; the proof of
 * possession (HMAC-SHA256 over the nonce) is the last word before the
 * app's own hello opens the lane. Until that hello arrives, requests
 * queue — a request sent before the association completes would be a
 * protocol violation, and violations are answered with silence.
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
  type FaceKind,
  type HostMessage,
  PROTOCOL_VERSION,
  type RpcRequest,
  type RpcResponse
} from "@castellan/protocol";
import {
  type AssociationVault,
  type KeyFactory,
  labelForFace,
  loadOrCreateAssociation,
  type Prover,
  webcryptoKeyFactory,
  webcryptoProver
} from "@/src/association";
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

/**
 * Who this extension is, for the association handshake. Every part is
 * injectable so the handshake's ordering can be tested deterministically;
 * the defaults are the production wiring (WebCrypto, a vault in
 * browser.storage.local).
 */
export interface ClientIdentity {
  /** Which browser build this is, for the app's panel. */
  face: FaceKind;
  /** The extension's own version, for the app's panel. */
  clientVersion: string;
  /** Where the association key lives between connections. */
  association: AssociationVault;
  /** How to answer a nonce challenge. */
  prove?: Prover;
  /** How to generate a first key. */
  createKey?: KeyFactory;
}

export function nativeMessagingTransport(
  getRuntime: () => NativeMessaging,
  identity: ClientIdentity
): Transport {
  const prove = identity.prove ?? webcryptoProver;
  const createKey = identity.createKey ?? webcryptoKeyFactory;
  const pending = new Map<
    number,
    { resolve: (r: RpcResponse) => void; reject: (e: Error) => void }
  >();
  const listeners = new Set<(event: Event) => void>();
  /** Requests that arrived while the association handshake was still
   * running — sent, in order, the moment the app's hello opens the lane. */
  const queued: RpcRequest[] = [];

  let port: NativePort | undefined;
  let alive = false;
  /** The material behind the current connection's claim: the challenge
   * answer needs it, and it is never re-sent as material. */
  let keyHex: string | undefined;
  /** One re-enrollment per connection (see the unknown_key handler). */
  let unknownKeySeen = false;

  function failAll(cause: Error): void {
    for (const waiter of pending.values()) waiter.reject(cause);
    pending.clear();
    queued.length = 0;
  }

  function connect(): NativePort {
    const fresh = getRuntime().connectNative(HOST_NAME);

    fresh.onMessage.addListener((message: HostMessage) => {
      if (port !== fresh) return; // a stale port's late message is noise

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
        return;
      }
      if (message.kind === "challenge") {
        // Answer the nonce with a proof over the claimed key. `void`: the
        // posting is fire-and-forget, and a rejected proof means the port
        // is going down anyway (the app answers failures with silence).
        void prove(keyHex ?? "", message.nonce_hex).then((proof_hex) => {
          if (port !== fresh) return;
          fresh.postMessage({
            kind: "proof",
            key_id: message.key_id,
            proof_hex
          } satisfies ClientMessage);
        });
        return;
      }
      if (message.kind === "unknown_key") {
        // The app forgot this key (a wiped store): enroll again with the
        // material we still hold. Exactly once — a second unknown_key on
        // one connection is the app refusing, not forgetting.
        if (keyHex === undefined || unknownKeySeen) return;
        unknownKeySeen = true;
        fresh.postMessage({
          kind: "hello",
          protocol_version: PROTOCOL_VERSION,
          face: identity.face,
          client_version: identity.clientVersion,
          association: {
            kind: "enroll",
            key_id: message.key_id,
            key_hex: keyHex,
            label: labelForFace(identity.face)
          }
        } satisfies ClientMessage);
        return;
      }
      // kind === "hello": the app's answer. The lane is open — release
      // everything that queued during the association handshake.
      alive = true;
      for (const request of queued.splice(0)) {
        fresh.postMessage({ kind: "request", request } satisfies ClientMessage);
      }
    });

    fresh.onDisconnect.addListener(() => {
      // A late disconnect from an old port must not tear down a replacement
      // connection or reject requests already sent over that replacement.
      if (port !== fresh) return;
      port = undefined;
      alive = false;
      keyHex = undefined;
      unknownKeySeen = false;
      // Every waiting caller learns the app went away now, not on a timer.
      failAll(disconnected("the native host disconnected"));
    });

    port = fresh;
    alive = false;
    unknownKeySeen = false;
    // Exactly one hello per physical connection, carrying the association
    // claim — a claim when the app already knows this key, an enrollment
    // when it might not. Reconnect redoes the whole handshake.
    void loadOrCreateAssociation(identity.association, createKey, identity.face).then(
      ({ claim, key_hex }) => {
        if (port !== fresh) return; // superseded while loading
        keyHex = key_hex;
        fresh.postMessage({
          kind: "hello",
          protocol_version: PROTOCOL_VERSION,
          face: identity.face,
          client_version: identity.clientVersion,
          association: claim
        } satisfies ClientMessage);
      },
      // A storage failure is a transport failure: nothing can be sent
      // without a claim, so treat it as the port never having opened.
      (cause: unknown) => {
        if (port !== fresh) return;
        port = undefined;
        failAll(disconnected(`the association key could not be loaded: ${String(cause)}`));
      }
    );
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
          // A burst during the handshake must share the connection the
          // handshake is running on: aliveness gates *sending*, port
          // existence gates *connecting*. (Gating both on aliveness would
          // open one port per queued request, each killing the previous
          // handshake's hello — the bug the burst property exists to pin.)
          const target = port ?? connect();
          if (alive) {
            target.postMessage({ kind: "request", request: req } satisfies ClientMessage);
          } else {
            // The association handshake owns the wire until the app's
            // hello; queue, and let the handshake flush in order.
            queued.push(req);
          }
        } catch (cause) {
          port = undefined;
          alive = false;
          const failure = disconnected(String(cause));
          failAll(failure);
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
