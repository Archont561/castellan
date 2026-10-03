/**
 * The generated types are the wire: what serde_json writes on the Rust side
 * is what these types describe, and each side parses what the other wrote.
 * This file is the TypeScript mirror of the crates' proptest round-trips
 * (see `arbitrary_payloads_round_trip` in castellan-ipc): every generated
 * type, fed arbitrary values, must survive a JSON round-trip unchanged.
 *
 * That property is exactly what keeps the wire honest. A u64 that ts-rs
 * decides to emit as `bigint`, a Date that stringifies, a field that only
 * sometimes survives the trip — any of those breaks this test before it
 * breaks a user's fill dialog. And because each arbitrary is typed as the
 * generated type it builds, a drift in the generated shape fails
 * compilation here, not a release there.
 *
 * The unions (`Capability`, `VaultStatus`) carry no structure of their own;
 * they are exercised through `Hello` and `RpcMethod`, which contain them.
 */

import { describe, expect, test } from "bun:test";
import fc from "fast-check";

import type {
  ClientMessage,
  EntrySummary,
  Event,
  Hello,
  HostMessage,
  NewEntry,
  RpcError,
  RpcErrorCode,
  RpcMethod,
  RpcRequest,
  RpcResponse,
  RpcResult
} from "@/src/index";

/** Text small enough to shrink readably; strings are the whole story here,
 * and fast-check's default alphabet already covers the hostile cases
 * (escapes, quotes, surrogates) that JSON.stringify must survive. */
const text = fc.string({ minLength: 0, maxLength: 24 });
const nullableText = fc.oneof(text, fc.constant(null));

const arbEntrySummary: fc.Arbitrary<EntrySummary> = fc.record({
  id: text,
  title: text,
  username: nullableText,
  url: nullableText,
  has_totp: fc.boolean(),
  has_passkey: fc.boolean()
});

const arbNewEntry: fc.Arbitrary<NewEntry> = fc.record({
  title: text,
  username: nullableText,
  password: nullableText,
  url: nullableText,
  otpauth: nullableText
});

const arbRpcErrorCode: fc.Arbitrary<RpcErrorCode> = fc.constantFrom(
  "vault_locked",
  "no_such_entry",
  "not_implemented"
);

const arbRpcError: fc.Arbitrary<RpcError> = fc.record({
  code: arbRpcErrorCode,
  message: text
});

const arbEvent: fc.Arbitrary<Event> = fc.oneof(
  fc.record({ type: fc.constant("database_locked") }),
  fc.record({ type: fc.constant("database_unlocked") }),
  fc.record({ type: fc.constant("entry_changed"), id: text })
);

const arbHello: fc.Arbitrary<Hello> = fc.record({
  protocol_version: fc.nat(),
  app_version: text,
  vault: fc.constantFrom("unlocked", "locked", "no_database"),
  capabilities: fc.uniqueArray(
    fc.constantFrom("autofill", "totp", "passkeys", "recovery_codes", "beam")
  )
});

/** Build a union arbitrary without losing its member type through
 * `Object.values`. The explicit empty check also documents the invariant the
 * mapped registries below rely on: an RPC contract has at least one member. */
function oneOf<T>(arbitraries: readonly fc.Arbitrary<T>[]): fc.Arbitrary<T> {
  const [first, ...rest] = arbitraries;
  if (first === undefined) throw new Error("cannot build an empty arbitrary union");
  return fc.oneof(first, ...rest);
}

type MethodArbitraries = {
  [Method in RpcMethod["method"]]: fc.Arbitrary<Extract<RpcMethod, { method: Method }>>;
};

// `satisfies MethodArbitraries` is the coverage gate: adding an operation to
// the Rust macro adds a generated discriminant and makes this object fail to
// compile until that operation receives an arbitrary.
const methodArbitraries = {
  get_entries: fc.record({ method: fc.constant("get_entries"), origin: text }),
  get_totp: fc.record({ method: fc.constant("get_totp"), entry_id: text }),
  generate_passphrase: fc.record({
    method: fc.constant("generate_passphrase"),
    words: fc.nat(64),
    separator: text
  }),
  save_entry: fc.record({ method: fc.constant("save_entry"), entry: arbNewEntry }),
  lock_database: fc.record({ method: fc.constant("lock_database") }),
  ping: fc.record({ method: fc.constant("ping") })
} satisfies MethodArbitraries;

const arbRpcMethod: fc.Arbitrary<RpcMethod> = oneOf(
  Object.values(methodArbitraries) as fc.Arbitrary<RpcMethod>[]
);

// RpcRequest is exactly `{ id, ...method }` because Rust uses serde flatten.
// Deriving it here avoids a second operation registry while preserving that
// flat wire contract in every generated case.
const arbRpcRequest: fc.Arbitrary<RpcRequest> = fc
  .tuple(fc.nat(), arbRpcMethod)
  .map(([id, method]) => ({ id, ...method }));

type ResultArbitraries = {
  [Type in RpcResult["type"]]: fc.Arbitrary<Extract<RpcResult, { type: Type }>>;
};

const resultArbitraries = {
  get_entries: fc.record({ type: fc.constant("get_entries"), entries: fc.array(arbEntrySummary) }),
  get_totp: fc.record({ type: fc.constant("get_totp"), code: text, seconds_remaining: fc.nat() }),
  generate_passphrase: fc.record({ type: fc.constant("generate_passphrase"), value: text }),
  save_entry: fc.record({ type: fc.constant("save_entry"), id: text }),
  lock_database: fc.record({ type: fc.constant("lock_database") }),
  ping: fc.record({ type: fc.constant("ping") })
} satisfies ResultArbitraries;

const arbRpcResult: fc.Arbitrary<RpcResult> = oneOf(
  Object.values(resultArbitraries) as fc.Arbitrary<RpcResult>[]
);

const arbRpcResponse: fc.Arbitrary<RpcResponse> = fc.record({
  id: fc.nat(),
  result: fc.oneof(arbRpcResult, fc.constant(null)),
  error: fc.oneof(arbRpcError, fc.constant(null))
});

// The association handshake (protocol v3): a claim can enroll (material
// crosses once) or just name a key, and both the face and the whole claim
// are nullable so a v2 hello still parses.
const arbFace = fc.constantFrom(
  "chrome",
  "edge",
  "brave",
  "vivaldi",
  "firefox",
  "safari",
  "cli",
  "other"
);

const arbAssociationClaim = fc.oneof(
  fc.record({
    kind: fc.constant("enroll"),
    key_id: text,
    key_hex: text,
    label: text
  }),
  fc.record({ kind: fc.constant("claim"), key_id: text })
);

const arbClientMessage: fc.Arbitrary<ClientMessage> = fc.oneof(
  fc.record({
    kind: fc.constant("hello"),
    protocol_version: fc.nat(),
    face: fc.oneof(arbFace, fc.constant(null)),
    client_version: nullableText,
    association: fc.oneof(arbAssociationClaim, fc.constant(null))
  }),
  fc.record({ kind: fc.constant("proof"), key_id: text, proof_hex: text }),
  fc.record({ kind: fc.constant("request"), request: arbRpcRequest })
);

const arbHostMessage: fc.Arbitrary<HostMessage> = fc.oneof(
  fc.record({ kind: fc.constant("hello"), hello: arbHello }),
  fc.record({ kind: fc.constant("challenge"), key_id: text, nonce_hex: text }),
  fc.record({ kind: fc.constant("unknown_key"), key_id: text }),
  fc.record({ kind: fc.constant("response"), response: arbRpcResponse }),
  fc.record({ kind: fc.constant("event"), event: arbEvent })
);

/** The types whose JSON transparency is the whole protocol, one entry per
 * generated type. One test per entry, so a failure names the type that
 * broke and fast-check shrinks inside that type only. */
const wireTypes: ReadonlyArray<[name: string, arb: fc.Arbitrary<unknown>]> = [
  ["EntrySummary", arbEntrySummary],
  ["NewEntry", arbNewEntry],
  ["RpcErrorCode", arbRpcErrorCode],
  ["RpcError", arbRpcError],
  ["Event", arbEvent],
  ["Hello", arbHello],
  ["RpcMethod", arbRpcMethod],
  ["RpcRequest", arbRpcRequest],
  ["RpcResult", arbRpcResult],
  ["RpcResponse", arbRpcResponse],
  ["ClientMessage", arbClientMessage],
  ["HostMessage", arbHostMessage]
];

describe("@castellan/protocol (properties)", () => {
  for (const [name, arb] of wireTypes) {
    test(`${name} survives a JSON round-trip unchanged`, () => {
      fc.assert(
        fc.property(arb, (value) => {
          expect(JSON.parse(JSON.stringify(value))).toEqual(value);
        })
      );
    });
  }
});
