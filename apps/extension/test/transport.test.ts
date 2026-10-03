import { describe, expect, test } from "bun:test";
import { type ClientMessage, type HostMessage, PROTOCOL_VERSION } from "@castellan/protocol";
import { createFixture } from "@castellan/utils/fixtures";
import fc from "fast-check";

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

const ping = (id: number) => ({ id, method: "ping" as const });
const pingResponse = (id: number): Extract<HostMessage, { kind: "response" }> => ({
  kind: "response",
  response: { id, result: { type: "ping" }, error: null }
});

/**
 * A transport over a fake runtime, fresh per test — the wiring every test
 * in this file needs, built once. `send` opens (or reuses) the connection
 * the way production code does, by sending a request, and hands back the
 * port the traffic traveled over; the properties below construct the same
 * pair per `fc.assert` run, where a per-test fixture cannot reach.
 */
const nativeHost = createFixture(() => {
  const runtime = new FakeRuntime();
  const transport = nativeMessagingTransport(() => runtime);
  return {
    runtime,
    transport,
    send: (...requests: Array<{ id: number; method: "ping" }>) => {
      const pending = requests.map((request) => transport.request(request));
      const port = runtime.ports.at(-1);
      if (port === undefined) throw new Error("transport did not open a port");
      return { port, pending };
    }
  };
});

describe("nativeMessagingTransport", () => {
  test("uses the generated host and sends one current-version hello per connection", async () => {
    const { send } = nativeHost();

    const { port, pending } = send(ping(1));
    expect(port.sent).toEqual([
      { kind: "hello", protocol_version: PROTOCOL_VERSION },
      { kind: "request", request: ping(1) }
    ]);
    port.onMessage.emit(pingResponse(1));
    await expect(pending[0]).resolves.toEqual(pingResponse(1).response);

    const second = send(ping(2));
    expect(second.port).toBe(port);
    expect(port.sent).toEqual([
      { kind: "hello", protocol_version: PROTOCOL_VERSION },
      { kind: "request", request: ping(1) },
      { kind: "request", request: ping(2) }
    ]);
    port.onMessage.emit(pingResponse(2));
    await second.pending[0];
  });

  test("rejects pending calls on disconnect and handshakes again after reconnect", async () => {
    const { transport, send } = nativeHost();

    const { port: firstPort, pending } = send(ping(1));
    firstPort.onDisconnect.emit();

    await expect(pending[0]).rejects.toMatchObject({ code: "disconnected" });
    expect(transport.connected).toBe(false);

    const retried = send(ping(2));
    const secondPort = retried.port;
    expect(secondPort).not.toBe(firstPort);
    expect(secondPort.sent[0]).toEqual({ kind: "hello", protocol_version: PROTOCOL_VERSION });
    firstPort.onDisconnect.emit();
    expect(transport.connected).toBe(true);
    secondPort.onMessage.emit(pingResponse(2));
    await retried.pending[0];
  });

  test("fans out events and honors unsubscribe", () => {
    const { transport, send } = nativeHost();
    const seen: HostMessage[] = [];
    const stop = transport.onEvent((event) => seen.push({ kind: "event", event }));

    const { port } = send(ping(1));
    port.onMessage.emit({ kind: "event", event: { type: "database_locked" } });
    stop();
    port.onMessage.emit({ kind: "event", event: { type: "database_unlocked" } });

    expect(seen).toEqual([{ kind: "event", event: { type: "database_locked" } }]);
  });

  test("any burst shares one connection, one hello, and answers by id in any order", async () => {
    await fc.assert(
      fc.asyncProperty(fc.nat(19), async (spread) => {
        const runtime = new FakeRuntime();
        const transport = nativeMessagingTransport(() => runtime);
        const calls = spread + 1;

        const requests = Array.from({ length: calls }, (_, index) => ping(index + 1));
        const pending = requests.map((request) => transport.request(request));
        const port = runtime.ports[0];
        if (port === undefined) throw new Error("transport did not open a port");

        // One handshake for the whole burst, then every request, in order —
        // the connection is a lane, not a letter per call.
        expect(port.sent).toEqual([
          { kind: "hello", protocol_version: PROTOCOL_VERSION },
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

  test("a disconnect rejects every pending call, whatever the burst", async () => {
    await fc.assert(
      fc.asyncProperty(fc.nat(9), async (spread) => {
        const runtime = new FakeRuntime();
        const transport = nativeMessagingTransport(() => runtime);
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
