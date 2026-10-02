import { describe, expect, test } from "bun:test";
import { type ClientMessage, type HostMessage, PROTOCOL_VERSION } from "@castellan/protocol";

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

describe("nativeMessagingTransport", () => {
  test("uses the generated host and sends one current-version hello per connection", async () => {
    const runtime = new FakeRuntime();
    const transport = nativeMessagingTransport(() => runtime);

    const first = transport.request(ping(1));
    const port = runtime.ports[0];
    if (port === undefined) throw new Error("transport did not open a port");

    expect(runtime.names).toEqual([HOST_NAME]);
    expect(port.sent).toEqual([
      { kind: "hello", protocol_version: PROTOCOL_VERSION },
      { kind: "request", request: ping(1) }
    ]);
    port.onMessage.emit(pingResponse(1));
    await expect(first).resolves.toEqual(pingResponse(1).response);

    const second = transport.request(ping(2));
    expect(port.sent).toEqual([
      { kind: "hello", protocol_version: PROTOCOL_VERSION },
      { kind: "request", request: ping(1) },
      { kind: "request", request: ping(2) }
    ]);
    port.onMessage.emit(pingResponse(2));
    await second;
  });

  test("rejects pending calls on disconnect and handshakes again after reconnect", async () => {
    const runtime = new FakeRuntime();
    const transport = nativeMessagingTransport(() => runtime);

    const interrupted = transport.request(ping(1));
    const firstPort = runtime.ports[0];
    if (firstPort === undefined) throw new Error("transport did not open a port");
    firstPort.onDisconnect.emit();

    await expect(interrupted).rejects.toMatchObject({ code: "disconnected" });
    expect(transport.connected).toBe(false);

    const retried = transport.request(ping(2));
    const secondPort = runtime.ports[1];
    if (secondPort === undefined) throw new Error("transport did not reconnect");
    expect(secondPort.sent[0]).toEqual({ kind: "hello", protocol_version: PROTOCOL_VERSION });
    firstPort.onDisconnect.emit();
    expect(transport.connected).toBe(true);
    secondPort.onMessage.emit(pingResponse(2));
    await retried;
  });

  test("fans out events and honors unsubscribe", () => {
    const runtime = new FakeRuntime();
    const transport = nativeMessagingTransport(() => runtime);
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
});
