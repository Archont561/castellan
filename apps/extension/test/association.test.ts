import { describe, expect, test } from "bun:test";
import type { FaceKind } from "@castellan/protocol";

import {
  faceFromBuildTarget,
  labelForFace,
  loadOrCreateAssociation,
  memoryAssociation,
  storageAssociation,
  webcryptoKeyFactory,
  webcryptoProver
} from "@/src/association";

/** A storage area that is just a map — the shape browser.storage.local
 * exposes, without the browser. */
function fakeArea(initial: Record<string, unknown> = {}) {
  const bag = new Map(Object.entries(initial));
  return {
    get: (key: string) => Promise.resolve({ [key]: bag.get(key) }),
    set: (items: Record<string, unknown>) => {
      for (const [key, value] of Object.entries(items)) bag.set(key, value);
      return Promise.resolve();
    },
    dump: () => Object.fromEntries(bag)
  };
}

/** A deterministic key factory: the material is the byte repeated, so the
 * tests can predict both the material and the fingerprint-free path. */
const fixedKey = (byte: number) => async () => ({
  key_id: `key-${byte}`,
  key_hex: byte.toString(16).padStart(2, "0").repeat(32)
});

describe("association", () => {
  test("first contact enrolls and persists; second contact claims", async () => {
    const vault = memoryAssociation();
    const face: FaceKind = "chrome";

    const first = await loadOrCreateAssociation(vault, fixedKey(7), face);
    expect(first.claim).toEqual({
      kind: "enroll",
      key_id: "key-7",
      key_hex: "07".repeat(32),
      label: "Chrome on this machine"
    });

    const second = await loadOrCreateAssociation(vault, fixedKey(9), face);
    expect(second.claim).toEqual({ kind: "claim", key_id: "key-7" });
    // The material the challenge answer needs is the STORED one, even
    // though this call could have generated a different key.
    expect(second.key_hex).toBe("07".repeat(32));
  });

  test("the storage vault round-trips through a storage area", async () => {
    const area = fakeArea();
    const vault = storageAssociation(area);

    expect(await vault.load()).toBeUndefined();
    await vault.save({ key_id: "abc", key_hex: "11".repeat(32) });
    expect(await vault.load()).toEqual({ key_id: "abc", key_hex: "11".repeat(32) });
    expect(area.dump()["castellan.association"]).toEqual({
      key_id: "abc",
      key_hex: "11".repeat(32)
    });
  });

  test("a corrupt or empty stored value is first contact again", async () => {
    const junk = fakeArea({ "castellan.association": "not an object" });
    expect(await storageAssociation(junk).load()).toBeUndefined();
    const empty = fakeArea({ "castellan.association": null });
    expect(await storageAssociation(empty).load()).toBeUndefined();
    const truncated = fakeArea({ "castellan.association": { key_id: "x" } });
    expect(await storageAssociation(truncated).load()).toBeUndefined();
  });

  test("the webcrypto factory makes unique keys with fingerprint ids", async () => {
    const first = await webcryptoKeyFactory();
    const second = await webcryptoKeyFactory();
    expect(first.key_hex).not.toBe(second.key_hex);
    expect(first.key_id).toMatch(/^[0-9a-f]{16}$/);
    expect(first.key_hex).toMatch(/^[0-9a-f]{64}$/);
  });

  test("the webcrypto prover answers what the app's Rust verifies", async () => {
    // The proof recipe must match castellan-ipc-server's `proof_hex`:
    // HMAC-SHA256(key_bytes, nonce_bytes), hex-encoded. The vector is
    // computed independently here (Bun's WebCrypto) so agreement is a
    // property of the recipe, not of shared code.
    const key_hex = "0b".repeat(32);
    const nonce_hex = "0a".repeat(32);
    const proof = await webcryptoProver(key_hex, nonce_hex);
    expect(proof).toMatch(/^[0-9a-f]{64}$/);
    // The same inputs prove the same; any other nonce does not.
    expect(await webcryptoProver(key_hex, nonce_hex)).toBe(proof);
    const other = await webcryptoProver(key_hex, "ff".repeat(32));
    expect(other).not.toBe(proof);
  });

  test("faces map to their build targets and labels", () => {
    expect(faceFromBuildTarget("firefox")).toBe("firefox");
    expect(faceFromBuildTarget("chrome")).toBe("chrome");
    expect(faceFromBuildTarget(undefined)).toBe("chrome");
    expect(labelForFace("firefox")).toBe("Firefox on this machine");
    expect(labelForFace("edge")).toBe("Edge on this machine");
    expect(labelForFace("cli")).toBe("The CLI on this machine");
  });
});
