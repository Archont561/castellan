---
id: TASK-3
title: "OTP crate — otpauth parsing and RFC 6238 TOTP"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 19:10'
updated_date: '2026-10-01 20:00'
labels:
  - otp
  - crypto
dependencies: []
references:
  - crates/otp/src/lib.rs
priority: high
ordinal: 300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Parse otpauth://totp URIs by hand (unregistered scheme, parsers disagree), compute RFC 6238 SHA-1 codes with dynamic truncation, refuse non-SHA1 algorithms with clear errors rather than wrong codes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 RFC 6238 Appendix B vectors pass (T=59 → 287082, T=1111111109 → 081804)
- [x] #2 `algorithm=SHA256`/`encoder=steam` are refused as UnsupportedParameter, not ignored
- [x] #3 `code_now` returns remaining seconds so the UI never reimplements period arithmetic
- [x] #4 Percent-decoding covers the escapes that actually occur in otpauth labels
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
std + hmac + sha1 + data-encoding only; parse by hand; expose code_at separately from code_now so tests pin vectors instead of freezing the clock.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
hmac 0.13 needs `KeyInit` imported for `new_from_slice`. The issuer parameter wins over the path issuer because that is what providers do in practice.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
5 tests green; the same crate feeds the apps natively and the extension through the WASM face.
<!-- SECTION:FINAL_SUMMARY:END -->
