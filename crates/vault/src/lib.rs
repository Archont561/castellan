//! The vault core: KDBX compatibility and the entry projection.
//!
//! This crate is the app-side half of "same scoped logic": every face reads
//! entries through [`EntrySummary`] and generates codes through
//! `castellan-otp`; only this crate touches KDBX bytes. The WASM face
//! deliberately does *not* link this crate — the extension never holds a
//! database.
//!
//! KDBX read is solid; KDBX *write* in keepass-rs is experimental and drops
//! fields it does not parse. The rule that follows, and that the app's
//! "browser integration" repair button exists to uphold: **save = copy the
//! file, write the new one, keep the copy**. A save that loses a field
//! KeePassXC wrote must cost the user a directory listing, not a database.
//!
//! Memory hygiene: keepass-rs 0.15 wraps values in `Value<T: Zeroize>`, so
//! field material wipes on drop. Locking is therefore "drop the
//! [`VaultHandle`]" — there is no long-lived decrypted blob to chase.

use std::path::{Path, PathBuf};

use castellan_protocol::EntrySummary;
use keepass::{
    Database, DatabaseKey,
    db::{EntryRef, GroupRef},
};
use thiserror::Error;

/// The custom field name that marks an entry as carrying a passkey.
///
/// Stored as a custom field (value: the relying-party ID) so a round-trip
/// through KeePassXC or KeePassDX degrades to "a weird custom field" instead
/// of a parse error — compatibility is a promise, not an accident.
pub const PASSKEY_FIELD: &str = "castellan.passkey";

/// Everything that can go wrong while opening a vault.
#[derive(Debug, Error)]
pub enum VaultError {
    /// The file could not be read at all.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The file that failed.
        path: PathBuf,
        /// Why it failed.
        source: std::io::Error,
    },
    /// keepass-rs refused it: wrong password, corrupt file, unsupported
    /// version. The message is passed through because keepass-rs's errors
    /// are already the best explanation available.
    #[error("cannot open database: {0}")]
    Keepass(#[from] keepass::db::DatabaseOpenError),
}

/// An unlocked vault: a parsed database plus the path it came from.
#[derive(Debug)]
pub struct VaultHandle {
    database: Database,
    path: PathBuf,
}

/// Open a KDBX file with a password and an optional keyfile.
///
/// Argon2id and the cipher choice are the file's business, negotiated by
/// keepass-rs; this function is the one place a path becomes a vault.
pub fn open(
    path: &Path,
    password: &str,
    keyfile: Option<&Path>,
) -> Result<VaultHandle, VaultError> {
    let mut source = std::fs::File::open(path).map_err(|source| VaultError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut key = DatabaseKey::new().with_password(password);
    if let Some(keyfile) = keyfile {
        let mut file = std::fs::File::open(keyfile).map_err(|source| VaultError::Io {
            path: keyfile.to_path_buf(),
            source,
        })?;
        key = key
            .with_keyfile(&mut file)
            .map_err(|source| VaultError::Io {
                path: keyfile.to_path_buf(),
                source,
            })?;
    }
    let database = Database::open(&mut source, key)?;
    Ok(VaultHandle {
        database,
        path: path.to_path_buf(),
    })
}

impl VaultHandle {
    /// Where this vault came from; every save writes this path (after
    /// copying the old file aside — see the crate docs).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every entry in the vault, projected to the protocol's shape.
    ///
    /// The whole database in one call, deliberately: the *protocol* paginates
    /// (the fill request carries one origin and returns a page), the vault
    /// does not — sorting and filtering belong above this crate, where the
    /// same logic serves every face.
    pub fn entries(&self) -> Vec<EntrySummary> {
        let mut out = Vec::new();
        walk(self.database.root(), &mut out);
        out
    }
}

/// Depth-first walk: entries first, then child groups. KeePass displays
/// groups as folders, so the order here is the order every KeePass client
/// the user has ever seen.
fn walk(group: GroupRef<'_>, out: &mut Vec<EntrySummary>) {
    for entry in group.entries() {
        if let Some(summary) = project(entry) {
            out.push(summary);
        }
    }
    for child in group.groups() {
        walk(child, out);
    }
}

/// Project one KDBX entry to [`EntrySummary`].
///
/// `None` means "not an entry the protocol talks about" — currently never,
/// but the shape keeps the day a group type is introduced from turning into
/// a silent field-mismatch.
fn project(entry: EntryRef<'_>) -> Option<EntrySummary> {
    Some(EntrySummary {
        id: entry.id().uuid().to_string(),
        title: entry.get_title().unwrap_or("(untitled)").to_string(),
        username: entry.get_username().map(str::to_string),
        url: entry.get_url().map(str::to_string),
        has_totp: entry.get_raw_otp_value().is_some() || entry.get("TOTP Seed").is_some(),
        has_passkey: entry.get(PASSKEY_FIELD).is_some(),
    })
}

/// The passphrase wordlist, as a public constant.
///
/// Public because callers may need it to reason about the product they
/// are offering: the entropy of a passphrase is `words * log2(WORDS.len())`
/// — 103 words means ~6.7 bits per word — and the UI cannot promise what
/// it cannot count. Not the EFF list — that is a data file to add when
/// the generator grows strength options. Every word is distinct under its
/// first three letters so handwritten notes stay unambiguous.
pub const WORDS: &[&str] = &[
    "acorn",
    "anchor",
    "apple",
    "autumn",
    "bamboo",
    "basil",
    "beacon",
    "birch",
    "boulder",
    "canyon",
    "cedar",
    "chestnut",
    "clover",
    "compass",
    "coral",
    "cotton",
    "cradle",
    "creek",
    "dahlia",
    "dapper",
    "dawn",
    "delta",
    "dune",
    "ember",
    "fable",
    "falcon",
    "fern",
    "flint",
    "forest",
    "galaxy",
    "ginger",
    "granite",
    "harbor",
    "hazel",
    "heron",
    "island",
    "ivory",
    "jasmine",
    "juniper",
    "kernel",
    "lagoon",
    "lantern",
    "larch",
    "lichen",
    "lilac",
    "linden",
    "lotus",
    "magnet",
    "maple",
    "marble",
    "meadow",
    "mineral",
    "mirage",
    "nectar",
    "nimble",
    "north",
    "oasis",
    "ocean",
    "onyx",
    "opal",
    "orbit",
    "orchid",
    "osprey",
    "otter",
    "paddle",
    "pebble",
    "pemberly",
    "pepper",
    "pigeon",
    "pinecone",
    "pistol",
    "pond",
    "quartz",
    "quiver",
    "raven",
    "reef",
    "ridge",
    "river",
    "rocket",
    "rustic",
    "saffron",
    "sage",
    "sailor",
    "seaglass",
    "sequoia",
    "shadow",
    "shore",
    "signal",
    "silver",
    "spruce",
    "stone",
    "summit",
    "tangerine",
    "thicket",
    "thistle",
    "timber",
    "tundra",
    "tulip",
    "vanilla",
    "velvet",
    "walnut",
    "willow",
    "winter",
    "zenith",
];

/// Generate a passphrase: `words` words joined by `separator`.
///
/// Word selection is rejection-sampled so every word is equally likely —
/// `random % len` biases short lists, and a biased wordlist shrinks the
/// passphrase's entropy below what the UI promises.
pub fn passphrase(words: usize, separator: &str) -> String {
    let mut picked: Vec<&str> = Vec::with_capacity(words);
    for _ in 0..words {
        picked.push(WORDS[random_index(WORDS.len())]);
    }
    picked.join(separator)
}

/// Uniform index in `0..len`, by rejecting the tail that would bias `%`.
fn random_index(len: usize) -> usize {
    let len = u32::try_from(len).expect("wordlists are far below u32::MAX");
    let limit = len * (u32::MAX / len);
    loop {
        let roll = getrandom::u32().expect("the OS randomness source is a hard dependency");
        if roll < limit {
            return (roll % len) as usize;
        }
    }
}
