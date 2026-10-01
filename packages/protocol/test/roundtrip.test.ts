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

const arbRpcError: fc.Arbitrary<RpcError> = fc.record({
  code: text,
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
  vault: fc.constantFrom("Unlocked", "Locked", "NoDatabase"),
  capabilities: fc.uniqueArray(
    fc.constantFrom("Autofill", "Totp", "Passkeys", "RecoveryCodes", "Beam")
  )
});

// RpcMethod and RpcRequest are one shape twice: the request is the method
// spread flat next to the id (`{ id, ...method }` — serde's `flatten`), so
// the request arbitraries below repeat the method fields per variant
// instead of nesting. That flatness is the wire contract; these records
// keep it from drifting.
const arbRpcMethod: fc.Arbitrary<RpcMethod> = fc.oneof(
  fc.record({ method: fc.constant("get_entries"), origin: text }),
  fc.record({ method: fc.constant("get_totp"), entry_id: text }),
  fc.record({ method: fc.constant("generate_passphrase"), words: fc.nat(64), separator: text }),
  fc.record({ method: fc.constant("save_entry"), entry: arbNewEntry }),
  fc.record({ method: fc.constant("lock_database") }),
  fc.record({ method: fc.constant("ping") })
);

const arbRpcRequest: fc.Arbitrary<RpcRequest> = fc.oneof(
  fc.record({ id: fc.nat(), method: fc.constant("get_entries"), origin: text }),
  fc.record({ id: fc.nat(), method: fc.constant("get_totp"), entry_id: text }),
  fc.record({
    id: fc.nat(),
    method: fc.constant("generate_passphrase"),
    words: fc.nat(64),
    separator: text
  }),
  fc.record({ id: fc.nat(), method: fc.constant("save_entry"), entry: arbNewEntry }),
  fc.record({ id: fc.nat(), method: fc.constant("lock_database") }),
  fc.record({ id: fc.nat(), method: fc.constant("ping") })
);

const arbRpcResult: fc.Arbitrary<RpcResult> = fc.oneof(
  fc.record({ type: fc.constant("ok") }),
  fc.record({ type: fc.constant("entries"), entries: fc.array(arbEntrySummary) }),
  fc.record({ type: fc.constant("totp"), code: text, seconds_remaining: fc.nat() }),
  fc.record({ type: fc.constant("passphrase"), value: text }),
  fc.record({ type: fc.constant("saved"), id: text })
);

const arbRpcResponse: fc.Arbitrary<RpcResponse> = fc.record({
  id: fc.nat(),
  result: fc.oneof(arbRpcResult, fc.constant(null)),
  error: fc.oneof(arbRpcError, fc.constant(null))
});

const arbClientMessage: fc.Arbitrary<ClientMessage> = fc.oneof(
  fc.record({ kind: fc.constant("hello"), protocol_version: fc.nat() }),
  fc.record({ kind: fc.constant("request"), request: arbRpcRequest })
);

const arbHostMessage: fc.Arbitrary<HostMessage> = fc.oneof(
  fc.record({ kind: fc.constant("hello"), hello: arbHello }),
  fc.record({ kind: fc.constant("response"), response: arbRpcResponse }),
  fc.record({ kind: fc.constant("event"), event: arbEvent })
);

/** The types whose JSON transparency is the whole protocol, one entry per
 * generated type. One test per entry, so a failure names the type that
 * broke and fast-check shrinks inside that type only. */
const wireTypes: ReadonlyArray<[name: string, arb: fc.Arbitrary<unknown>]> = [
  ["EntrySummary", arbEntrySummary],
  ["NewEntry", arbNewEntry],
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
