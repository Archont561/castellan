# `castellan-vault`

The KDBX vault core: opening a KeePass-compatible database, projecting safe
entry summaries, generating passphrases, managing the in-memory session and
lock tiers, importing TOTP records, and saving without risking the original
file.

## Security and persistence boundary

`VaultHandle` owns a decrypted database only in memory. `VaultSession` adds
lock policy, event emission, and the app-facing session lifecycle. Entry
projections deliberately omit secrets; TOTP seed retrieval happens only for a
selected entry. The passkey field has one named constant,
`PASSKEY_FIELD`, rather than an ad-hoc string in a UI.

Saving is intentionally copy-aside then write: `VaultHandle::save` writes a
temporary replacement only after preserving the old KDBX as a timestamped
`.bak`. Do not call Keepass save APIs directly from another crate or app shell;
that would bypass the recovery guarantee.

## Tests and fixtures

Integration tests use the public API. The fixture corpus is generated from
reviewed case tables except where external authorship or an unsupported write
format makes a committed anchor necessary; see
[`tests/fixtures/README.md`](tests/fixtures/README.md).

## Verify

```console
$ pixi run cargo nextest run -p castellan-vault
$ pixi run cargo test --doc -p castellan-vault
$ pixi run cargo clippy -p castellan-vault --all-targets -- -D warnings
```
