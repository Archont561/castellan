# `castellan-otp`

Shared authenticator logic: `otpauth://` parsing, RFC 6238 TOTP calculation,
and import preview for standard text, Google migration QR, Aegis, and andOTP
payloads. Native faces call it through the dispatcher; the browser receives a
secret-free projection from `castellan-wasm`.

## Public API

- `parse()` and `TotpConfig` validate an otpauth URI.
- `code_at()` and `code_now()` compute RFC 6238 codes and remaining seconds.
- `preview()` returns reviewable `AccountCandidate` values for supported import
  payloads.
- `migration::parse_migration()` parses Google migration URIs.
- `otpauth_uri()` writes a canonical compatible URI.

Unsupported algorithms or invalid payloads are explicit `OtpError` values;
this crate must never silently generate a plausible but wrong code. It does not
open a vault or expose a UI.

## Verify

```console
$ pixi run cargo nextest run -p castellan-otp
$ pixi run cargo test --doc -p castellan-otp
$ pixi run cargo clippy -p castellan-otp --all-targets -- -D warnings
```
