/**
 * The passkey interception layer.
 *
 * Runs at document_start in the page's MAIN world and overrides the WebAuthn
 * entry points (navigator.credentials.create/get, PublicKeyCredential
 * methods) so a ceremony becomes a protocol request to the app instead of a
 * browser-builtin prompt. This is the same technique 1Password and Bitwarden
 * use for desktop passkeys; the RP ID binding that makes passkeys
 * phishing-resistant is enforced by the *app* (origin is attached to every
 * request), not by this script, so a compromised extension cannot ask for
 * github.com secrets from evil.example.
 *
 * Currently a guard rail, not an implementation: the override is deliberately
 * inert until the background transport reports a connected app, and the
 * page's original functions are preserved untouched.
 */
export default defineContentScript({
  matches: ["<all_urls>"],
  runAt: "document_start",
  world: "MAIN",
  main() {
    // TODO(passkeys): the full override + relay to the background script.
    // The order here matters and is the whole file's reason to exist:
    // document_start + MAIN world means the page receives an already-
    // patched navigator.credentials, never the original.
  }
});
