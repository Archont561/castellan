/**
 * The extension's half of the association handshake (doc-2 §3): one key,
 * generated on this machine, stored in the extension's own storage, proven
 * — never re-sent — on every later connection.
 *
 * The material crosses the wire exactly once, inside the enroll claim,
 * over the same-user local socket. From then on the app challenges with a
 * fresh nonce and this module answers with an HMAC-SHA256 over it; the
 * symmetric key never leaves the machine again.
 *
 * Every moving part is injectable: the storage area, the randomness, the
 * proof. Production wires in `browser.storage.local` and WebCrypto; the
 * tests wire in memory and a deterministic prover, which is the only way
 * the handshake's order can be asserted rather than hoped for.
 */

import type { AssociationClaim, FaceKind } from "@castellan/protocol";

/** A generated association key: its fingerprint and its material. */
export interface KeyMaterial {
  /** The stable identifier: a fingerprint of the material. */
  key_id: string;
  /** The material itself, hex-encoded. */
  key_hex: string;
}

/** Where the key lives between connections. */
export interface AssociationVault {
  /** The stored key, or `undefined` on first contact.
   * @returns the stored material, if any. */
  load(): Promise<KeyMaterial | undefined>;
  /**
   * Remember a key.
   * @param material the key to store.
   */
  save(material: KeyMaterial): Promise<void>;
}

/** The storage surface `storageAssociation` needs — the shape of
 * `browser.storage.local`, narrowed to the two calls used. */
export interface StorageArea {
  /** @param key the storage key to read.
   * @returns the stored values for the key. */
  get(key: string): Promise<Record<string, unknown>>;
  /**
   * @param items values to persist.
   */
  set(items: Record<string, unknown>): Promise<void>;
}

const STORAGE_KEY = "castellan.association";

/** An in-memory vault — tests, and any context without storage. */
export function memoryAssociation(
  initial?: KeyMaterial
): AssociationVault & { material(): KeyMaterial | undefined } {
  let held = initial;
  return {
    load: () => Promise.resolve(held),
    save: (material) => {
      held = material;
      return Promise.resolve();
    },
    material: () => held
  };
}

/** A vault over a WebExtension storage area (production wiring). */
export function storageAssociation(area: StorageArea): AssociationVault {
  return {
    load: async () => {
      const bag = await area.get(STORAGE_KEY);
      const value = bag[STORAGE_KEY];
      if (isKeyMaterial(value)) return value;
      return undefined;
    },
    save: (material) => area.set({ [STORAGE_KEY]: material })
  };
}

function isKeyMaterial(value: unknown): value is KeyMaterial {
  if (typeof value !== "object" || value === null) return false;
  const { key_id, key_hex } = value as Record<string, unknown>;
  return typeof key_id === "string" && typeof key_hex === "string" && key_hex.length > 0;
}

/**
 * Compute a challenge answer: HMAC-SHA256 over the nonce, keyed by the
 * material. The default uses WebCrypto, which exists in every context the
 * extension runs in (service worker, popup, tests).
 */
export type Prover = (key_hex: string, nonce_hex: string) => Promise<string>;

/** A fresh key: 32 random bytes, identified by the first 16 hex chars of
 * SHA-256 over the material's hex form. */
export type KeyFactory = () => Promise<KeyMaterial>;

export const webcryptoProver: Prover = async (key_hex, nonce_hex) => {
  const key = await crypto.subtle.importKey(
    "raw",
    toBytes(key_hex),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const tag = await crypto.subtle.sign("HMAC", key, toBytes(nonce_hex));
  return toHex(new Uint8Array(tag));
};

export const webcryptoKeyFactory: KeyFactory = async () => {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  const key_hex = toHex(bytes);
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(key_hex));
  const key_id = toHex(new Uint8Array(digest)).slice(0, 16);
  return { key_id, key_hex };
};

/**
 * Load the stored key, or generate and persist one — and shape the claim
 * the hello should carry: a claim when the app already knows this key, an
 * enrollment when it might not.
 *
 * @param vault where the key lives.
 * @param createKey how to make a new one.
 * @param face for the enrollment's human label.
 * @returns the claim to send, and the material behind it (kept client-side
 * for the challenge answer; it is never re-sent).
 */
export async function loadOrCreateAssociation(
  vault: AssociationVault,
  createKey: KeyFactory,
  face: FaceKind
): Promise<{ claim: AssociationClaim; key_hex: string }> {
  const stored = await vault.load();
  if (stored) {
    return { claim: { kind: "claim", key_id: stored.key_id }, key_hex: stored.key_hex };
  }
  const material = await createKey();
  await vault.save(material);
  return {
    claim: {
      kind: "enroll",
      key_id: material.key_id,
      key_hex: material.key_hex,
      label: labelForFace(face)
    },
    key_hex: material.key_hex
  };
}

/** The prompt's human label for a face: "Chrome on this machine". */
export function labelForFace(face: FaceKind): string {
  return `${faceLabel(face)} on this machine`;
}

function faceLabel(face: FaceKind): string {
  const labels: Record<FaceKind, string> = {
    chrome: "Chrome",
    edge: "Edge",
    brave: "Brave",
    vivaldi: "Vivaldi",
    firefox: "Firefox",
    safari: "Safari",
    cli: "The CLI",
    other: "A browser"
  };
  return labels[face];
}

/**
 * Which face a build of the extension is. WXT defines the build target at
 * compile time (`import.meta.env.BROWSER`); the chromium family is built
 * from one tree, so everything not Firefox runs as Chrome until the
 * per-browser builds of task-10 give Edge, Brave and Vivaldi their own.
 */
export function faceFromBuildTarget(target: string | undefined): FaceKind {
  if (target === "firefox") return "firefox";
  return "chrome";
}

function toBytes(hex: string): Uint8Array<ArrayBuffer> {
  // Backed by a plain ArrayBuffer (not ArrayBufferLike) because WebCrypto's
  // BufferSource demands it under TS's stricter byte-source types.
  const bytes = new Uint8Array(new ArrayBuffer(hex.length / 2));
  for (let index = 0; index < bytes.length; index++) {
    bytes[index] = Number.parseInt(hex.slice(index * 2, index * 2 + 2), 16);
  }
  return bytes;
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}
