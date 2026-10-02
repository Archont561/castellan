/** The extension-local bridge between popup/content contexts and background. */

/** Ask the background worker to prove that the native app answers RPC. */
export const PING_BACKGROUND = "castellan:ping" as const;

/** Every message currently accepted by the background worker. */
export type ExtensionMessage = typeof PING_BACKGROUND;

/** The correlated answer to {@link PING_BACKGROUND}. */
export type PingBackgroundResponse = { ok: true } | { ok: false; cause: string };

/** Runtime guard for browser messages, whose platform type is intentionally unknown. */
export function isPingBackground(message: unknown): message is typeof PING_BACKGROUND {
  return message === PING_BACKGROUND;
}

/** The part of browser.runtime the popup needs, injectable for tests. */
export interface MessageSender {
  sendMessage(message: ExtensionMessage): Promise<unknown>;
}

/** Send the typed ping without repeating its string or response assertion at a caller. */
export async function pingBackground(runtime: MessageSender): Promise<PingBackgroundResponse> {
  return (await runtime.sendMessage(PING_BACKGROUND)) as PingBackgroundResponse;
}
