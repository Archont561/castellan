import { describe, expect, test } from "bun:test";
import {
  type ClientMessage,
  type FaceKind,
  type Hello,
  type HostMessage,
  PROTOCOL_VERSION
} from "@castellan/protocol";
import { createFixture } from "@castellan/utils/fixtures";
import fc from "fast-check";

import { memoryAssociation } from "@/src/association";
import { HOST_NAME, nativeMessagingTransport } from "@/src/transport";

class EventHook<Args extends unknown[]> {
  readonly listeners = new Set<(...args: Args) => void>();

  addListener(listener: (...args: Args) => void): void {
    this.listeners.add(listener);
  }

  removeListener(listener: (...args: Args) => void): void {
    this.listeners.delete(listener);
  }

  emit(...args: Args): void {
    for (const listener of this.listeners) listener(...args);
  }
}

class FakePort {
  readonly sent: ClientMessage[] = [];
  readonly onMessage = new EventHook<[message: HostMessage]>();
  readonly onDisconnect = new EventHook<[]>();

  postMessage(message: ClientMessage): void {
    this.sent.push(message);
  }
}

class FakeRuntime {
  readonly names: string[] = [];
  readonly ports: FakePort[] = [];

  connectNative(name: string): FakePort {
    this.names.push(name);
    const port = new FakePort();
    this.ports.push(port);
    return port;
  }
}

const KEY_ID = "feedfacefeedface";
const KEY_HEX = "07".repeat(32);
const NONCE_HEX = "ab".repeat(32);
const PROOF_HEX = "cd".repeat(32);

const ping = (id: number) => ({ id, method: "ping" as const });
const pingResponse = (id: number): Extract<HostMessage, { kind: "response" }> => ({
  kind: "response",
  response: { id, result: { type: "ping" }, error: null }
});
const appHello: Hello = {
  protocol_version: PROTOCOL_VERSION,
  app_version: "test-app",
  vault: "locked",
  capabilities: []
};
const enrollHello = (face: FaceKind = "chrome"): Extract<ClientMessage, { kind: "hello" }> => ({
  kind: "hello",
  protocol_version: PROTOCOL_VERSION,
  face,
  client_version: "test-extension",
  association: {
    kind: "enroll",
    key_id: KEY_ID,
    key_hex: KEY_HEX,
    label: "Chrome on this machine"
  }
});
const claimHello = (): Extract<ClientMessage, { kind: "hello" }> => ({
  kind: "hello",
  protocol_version: PROTOCOL_VERSION,
  face: "chrome",
  client_version: "test-extension",
  association: { kind: "claim", key_id: KEY_ID }
});
const challenge = (): Extract<HostMessage, { kind: "challenge" }> => ({
  kind: "challenge",
  key_id: KEY_ID,
  nonce_hex: NONCE_HEX
});

/** Let the transport's async handshake steps (vault load, proof) run. */
const settle = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/**
 * A transport over a fake runtime, fresh per test. The handshake is
 * driven by `shake`: the transport sends its hello, the fake app
 * challenges, the transport proves, the fake app opens the lane. Tests
 * that want to assert the handshake itself use the port directly instead.
 */
const nativeHost = createFixture(() => {
  const runtime = new FakeRuntime();
  const vault = memoryAssociation();
  const transport = nativeMessagingTransport(() => runtime, {
    face: "chrome",
    clientVersion: "test-extension",
    association: vault,
    createKey: async () => ({ key_id: KEY_ID, key_hex: KEY_HEX }),
    prove: async (key_hex, nonce_hex) => {
      expect(key_hex).toBe(KEY_HEX);
      expect(nonce_hex).toBe(NONCE_HEX);
      return PROOF_HEX;
    }
  });
  return {
    runtime,
    transport,
    vault,
    /** Open the lane the way a real app would, on the newest port. */
    shake: async () => {
      await settle();
      const port = runtime.ports.at(-1);
      if (port === undefined) throw new Error("transport did not open a port");
      port.onMessage.emit(challenge());
      await settle();
      port.onMessage.emit({ kind: "hello", hello: appHello });
      await settle();
      return port;
    },
    send: (...requests: Array<{ id: number; method: "ping" }>) => {
      const pending = requests.map((request) => transport.request(request));
      const port = runtime.ports.at(-1);
      if (port === undefined) throw new Error("transport did not open a port");
      return { port, pending };
    }
  };
});

describe("nativeMessagingTransport", () => {
  test("the handshake enrolls once, then claims on every later connection", async () => {
    const { runtime, transport, send } = nativeHost();

    const { pending } = send(ping(1));
    const port = runtime.ports[0];
    if (port === undefined) throw new Error("transport did not open a port");
    // First contact: hello carries the enrollment (face, version, material).
    await settle();
    expect(port.sent[0]).toEqual(enrollHello());
    expect(transport.connected).toBe(false);

    // The app challenges; the transport proves possession — the material
    // itself is never re-sent.
    port.onMessage.emit(challenge());
    await settle();
    expect(port.sent[1]).toEqual({
      kind: "proof",
      key_id: KEY_ID,
      proof_hex: PROOF_HEX
    });

    // The app's hello opens the lane; the queued request finally flies.
    port.onMessage.emit({ kind: "hello", hello: appHello });
    await settle();
    expect(transport.connected).toBe(true);
    expect(port.sent[2]).toEqual({ kind: "request", request: ping(1) });
    port.onMessage.emit(pingResponse(1));
    await expect(pending[0]).resolves.toEqual(pingResponse(1).response);

    // A reconnect claims the remembered key: no material crosses again.
    port.onDisconnect.emit();
    const second = send(ping(2));
    const secondPort = second.port;
    await settle();
    expect(secondPort.sent[0]).toEqual(claimHello());
    secondPort.onMessage.emit(challenge());
    await settle();
    secondPort.onMessage.emit({ kind: "hello", hello: appHello });
    await settle();
    secondPort.onMessage.emit(pingResponse(2));
    await second.pending[0];
    expect(
      secondPort.sent.filter(
        (message) => message.kind === "hello" && message.association?.kind === "enroll"
      )
    ).toHaveLength(0);
  });

  test("an unknown key re-enrolls once on the same connection", async () => {
    const { runtime, transport } = nativeHost();

    // Open a connection. The call below never resolves — the fake app in
    // this test never answers it — so it is fired and forgotten; the
    // handshake is what this test drives by hand.
    void transport.request(ping(0)).catch(() => {});
    const port = runtime.ports[0];
    if (port === undefined) throw new Error("transport did not open a port");
    await settle();
    expect(port.sent[0]).toEqual(enrollHello());

    port.onMessage.emit({ kind: "unknown_key", key_id: KEY_ID });
    await settle();
    expect(port.sent[1]).toEqual(enrollHello());

    // A second unknown_key on the same connection is the app refusing,
    // not forgetting: no second enrollment, no loop.
    port.onMessage.emit({ kind: "unknown_key", key_id: KEY_ID });
    await settle();
    expect(port.sent.filter((message) => message.kind === "hello")).toHaveLength(2);
  });

  test("requests queue until the lane opens, then fly in order", async () => {
    const { runtime, send } = nativeHost();

    const { pending } = send(ping(1), ping(2), ping(3));
    const port = runtime.ports[0];
    if (port === undefined) throw new Error("transport did not open a port");
    await settle();
    // Nothing but the handshake on the wire before the app's hello.
    expect(port.sent).toEqual([enrollHello()]);

    port.onMessage.emit(challenge());
    await settle();
    port.onMessage.emit({ kind: "hello", hello: appHello });
    await settle();
    expect(port.sent.slice(2)).toEqual([
      { kind: "request", request: ping(1) },
      { kind: "request", request: ping(2) },
      { kind: "request", request: ping(3) }
    ]);

    for (let id = 3; id >= 1; id--) port.onMessage.emit(pingResponse(id));
    const responses = await Promise.all(pending);
    expect(responses.map((response) => response.id)).toEqual([1, 2, 3]);
  });

  test("rejects pending calls on disconnect and handshakes again after reconnect", async () => {
    const { transport, send } = nativeHost();

    const { port: firstPort, pending } = send(ping(1));
    await settle();
    firstPort.onDisconnect.emit();

    await expect(pending[0]).rejects.toMatchObject({ code: "disconnected" });
    expect(transport.connected).toBe(false);

    const retried = send(ping(2));
    const secondPort = retried.port;
    if (secondPort === undefined) throw new Error("transport did not open a port");
    expect(secondPort).not.toBe(firstPort);
    await settle();
    expect(secondPort.sent[0]).toEqual(claimHello());
    firstPort.onDisconnect.emit(); // a late disconnect from the dead port
    expect(transport.connected).toBe(false);
    secondPort.onMessage.emit(challenge());
    await settle();
    secondPort.onMessage.emit({ kind: "hello", hello: appHello });
    await settle();
    expect(transport.connected).toBe(true);
    secondPort.onMessage.emit(pingResponse(2));
    await retried.pending[0];
  });

  test("fans out events and honors unsubscribe", async () => {
    const runtime = new FakeRuntime();
    const transport = nativeMessagingTransport(() => runtime, {
      face: "chrome",
      clientVersion: "test-extension",
      association: memoryAssociation({ key_id: KEY_ID, key_hex: KEY_HEX }),
      createKey: async () => ({ key_id: KEY_ID, key_hex: KEY_HEX }),
      prove: async () => PROOF_HEX
    });
    const seen: HostMessage[] = [];
    const stop = transport.onEvent((event) => seen.push({ kind: "event", event }));

    void transport.request(ping(1));
    const port = runtime.ports[0];
    if (port === undefined) throw new Error("transport did not open a port");
    port.onMessage.emit({ kind: "event", event: { type: "database_locked" } });
    stop();
    port.onMessage.emit({ kind: "event", event: { type: "database_unlocked" } });

    expect(seen).toEqual([{ kind: "event", event: { type: "database_locked" } }]);
  });

  test("any burst shares one connection, one handshake, and answers by id in any order", async () => {
    await fc.assert(
      fc.asyncProperty(fc.nat(19), async (spread) => {
        const runtime = new FakeRuntime();
        const transport = nativeMessagingTransport(() => runtime, {
          face: "chrome",
          clientVersion: "test-extension",
          association: memoryAssociation({ key_id: KEY_ID, key_hex: KEY_HEX }),
          createKey: async () => ({ key_id: KEY_ID, key_hex: KEY_HEX }),
          prove: async () => PROOF_HEX
        });
        const calls = spread + 1;

        const requests = Array.from({ length: calls }, (_, index) => ping(index + 1));
        const pending = requests.map((request) => transport.request(request));
        const port = runtime.ports[0];
        if (port === undefined) throw new Error("transport did not open a port");
        await settle();

        // Before the lane opens, nothing but the handshake is on the wire.
        expect(port.sent).toEqual([claimHello()]);

        // The app challenges, the transport proves, the lane opens — and
        // only then does the whole burst fly, in order. The connection is
        // a lane, not a letter per call.
        port.onMessage.emit(challenge());
        await settle();
        expect(port.sent[1]).toEqual({
          kind: "proof",
          key_id: KEY_ID,
          proof_hex: PROOF_HEX
        });
        port.onMessage.emit({ kind: "hello", hello: appHello });
        await settle();
        expect(port.sent.slice(2)).toEqual([
          ...requests.map((request): ClientMessage => ({ kind: "request", request }))
        ]);
        expect(runtime.names).toEqual([HOST_NAME]);

        // Answers arriving in reverse order still land on their own
        // callers: the multiplexer is keyed by request id, not by queue
        // position — which is the whole reason it exists.
        for (let id = calls; id >= 1; id--) {
          port.onMessage.emit(pingResponse(id));
        }
        const responses = await Promise.all(pending);
        expect(responses.map((response) => response.id)).toEqual(
          Array.from({ length: calls }, (_, index) => index + 1)
        );
      })
    );
  });

  test("a disconnect during the handshake rejects every queued call, whatever the burst", async () => {
    await fc.assert(
      fc.asyncProperty(fc.nat(9), async (spread) => {
        const runtime = new FakeRuntime();
        const transport = nativeMessagingTransport(() => runtime, {
          face: "chrome",
          clientVersion: "test-extension",
          association: memoryAssociation(),
          createKey: async () => ({ key_id: KEY_ID, key_hex: KEY_HEX }),
          prove: async () => PROOF_HEX
        });
        const calls = spread + 1;

        const pending = Array.from({ length: calls }, (_, index) =>
          transport.request(ping(index + 1))
        );
        const port = runtime.ports[0];
        if (port === undefined) throw new Error("transport did not open a port");
        port.onDisconnect.emit();

        for (const failing of pending) {
          await expect(failing).rejects.toMatchObject({ code: "disconnected" });
        }
        expect(transport.connected).toBe(false);
      })
    );
  });
});
