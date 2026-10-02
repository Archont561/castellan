/**
 * The package's public surface, proven once: the barrel resolves every
 * generated module, and the version it hands out is the one xtask generated
 * from the Rust constant — not a number someone updated by hand.
 *
 * There is deliberately nothing else here. This package is types and one
 * constant; behavior lives in @castellan/core, and a protocol package that
 * starts testing behavior is a protocol package that has drifted.
 */
import { describe, expect, test } from "bun:test";
import { PROTOCOL_VERSION as generatedVersion } from "@/src/generated/version";
import { PROTOCOL_VERSION } from "@/src/index";

describe("@castellan/protocol", () => {
  test("the barrel's constant is the generated one", () => {
    expect(PROTOCOL_VERSION).toBe(generatedVersion);
  });
});
