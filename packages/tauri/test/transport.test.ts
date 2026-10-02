import { describe, expect, test } from "bun:test";
import type { RpcResponse } from "@castellan/protocol";

import { createTauriTransport, type TauriInvoke } from "@/src/index";

describe("createTauriTransport", () => {
  test("sends the complete request through the single rpc command", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const invoke: TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => {
      calls.push(args === undefined ? { command } : { command, args });
      return Promise.resolve({ id: 7, result: { type: "ping" }, error: null } as T);
    };
    const transport = createTauriTransport(invoke);

    const response = await transport.request({ id: 7, method: "ping" });

    expect(calls).toEqual([{ command: "rpc", args: { request: { id: 7, method: "ping" } } }]);
    expect(response).toEqual({
      id: 7,
      result: { type: "ping" },
      error: null
    } satisfies RpcResponse);
  });

  test("normalizes invoke rejection as a disconnected transport error", async () => {
    const transport = createTauriTransport(() => Promise.reject(new Error("webview closed")));

    await expect(transport.request({ id: 1, method: "ping" })).rejects.toMatchObject({
      code: "disconnected"
    });
  });
});
