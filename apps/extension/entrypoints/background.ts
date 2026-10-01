/**
 * The background service worker: the extension's root.
 *
 * Owns one transport (native messaging to the app) and one client; the popup
 * and content scripts talk to *it*, never to the port directly, so the
 * connection strategy — connect, survive a service-worker sleep, reconnect —
 * exists in exactly one place.
 *
 * MV3 note: Chrome terminates idle service workers, which tears the native
 * port down. The transport reconnects on the next request; the only thing
 * this file adds is a wake-up alarm so a TOTP countdown in the popup does
 * not go stale between requests.
 */
import { CastellanClient } from "@castellan/core";
import { nativeMessagingTransport } from "@/src/transport";

export default defineBackground(() => {
  // WXT injects `browser` as a global, typed end to end (wxt/browser →
  // @wxt-dev/browser, types from @types/chrome): runtime.connectNative and
  // the port events are checked, no casts. The getter keeps the transport
  // testable — the transport takes a NativeMessaging it can be fed a fake
  // of, not the global itself.
  const client = new CastellanClient(nativeMessagingTransport(() => browser.runtime));

  // The popup asks; the background answers. Typed message passing (WXT's
  // defineExtensionMessaging) lands with the first real popup feature.
  browser.runtime.onMessage.addListener((message: unknown) => {
    if (message === "castellan:ping") {
      return client
        .ping()
        .then(() => ({ ok: true as const }))
        .catch((cause: unknown) => ({ ok: false as const, cause: String(cause) }));
    }
    return undefined;
  });
});
