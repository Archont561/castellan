//! The KDBX 4.1 fixture factory: one generator and a table of cases.
//!
//! The unlock corpus and the round-trip matrix used to be fourteen committed
//! binaries copied from keepass-rs's test resources. Most of them varied
//! only along axes our own stack can author — outer cipher, KDF, key
//! material, structural features — so they are now *generated* from the
//! case table below: reviewable as a table, exact in what the tests may
//! assert (the generator knows every title it wrote), and impossible to
//! drift out of sync with the parser.
//!
//! Two files stay committed in `tests/fixtures/`, because no generator can
//! author what they exist to prove:
//!
//! - `test_db_kdbx41_features.kdbx` — written by KeePassXC 2.7.12, the
//!   external-authority anchor: bytes a reader other than keepass-rs
//!   produced, which a keepass-rs-built fixture can never stand in for.
//! - `test_db_with_password.kdbx` — KDBX 3.1, the read-compatibility
//!   promise; keepass-rs's writer emits 4.1 only, so 3.1 cannot be built.
//!
//! The trade is deliberate and recorded in task-8's notes: generated cases
//! verify that *our* save path preserves config and content across the
//! matrix, while the anchors carry the "another client wrote this"
//! coverage.

// Each test binary (session, save, roundtrip) includes this module and
// uses the subset it needs; unused helpers in one binary are not drift.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use keepass::config::{CompressionConfig, DatabaseVersion, KdfConfig, OuterCipherConfig};
use keepass::{Database, DatabaseKey};

/// A unique scratch directory per call: tests run concurrently, and both
/// the copy-aside contract and Argon2-derived file state are exactly the
/// kind of thing a shared path would make flaky.
pub(crate) fn scratch(tag: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "castellan-kdbx41-{}-{tag}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Where the two committed anchor fixtures live.
pub(crate) fn anchor(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The shape of a generated keyfile. Both forms are what real KeePass
/// clients write; the bytes are fixed test constants, never secrets.
#[derive(Clone, Copy, Debug)]
pub(crate) enum KeyfileKind {
    /// The legacy raw form: exactly 32 bytes of key material.
    Raw32,
    /// The XML v2 form: `<KeyFile><Key><Data>base64</Data></Key></KeyFile>`.
    XmlV2,
}

/// What unlocks a generated vault.
#[derive(Clone, Copy, Debug)]
pub(crate) enum KeyMaterial {
    /// A password and nothing else.
    PasswordOnly(&'static str),
    /// No password; the keyfile is the whole key.
    KeyfileOnly {
        /// Which keyfile form to write.
        keyfile: KeyfileKind,
    },
    /// Both components present — the keyfile-everywhere shape.
    PasswordAndKeyfile {
        /// The password component.
        password: &'static str,
        /// Which keyfile form to write.
        keyfile: KeyfileKind,
    },
}

/// A structural feature a case turns on. The base content every case gets
/// (two entries, protected and unprotected fields, tags) is not listed —
/// only the things a named case isolates.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Feature {
    /// An entry carrying a raw `otp` otpauth URI (KeePassXC's convention).
    Totp,
    /// A recycle-bin group wired into meta (enabled + uuid) holding one
    /// entry — the soft-delete shape.
    RecycleBin,
    /// A uuid recorded in the deleted-objects map — the purged shape.
    DeletedObjectRecord,
    /// A history snapshot on the primary entry, built the way KeePass
    /// itself makes one: the entry's own earlier state, same uuid.
    History,
}

/// One generated corpus case: the config matrix row plus its features.
pub(crate) struct KdbxCase {
    /// The case's name — also the database name and file stem.
    pub(crate) name: &'static str,
    /// Outer cipher.
    pub(crate) cipher: OuterCipherConfig,
    /// Key derivation function.
    pub(crate) kdf: KdfConfig,
    /// Inner compression.
    pub(crate) compression: CompressionConfig,
    /// What unlocks the vault.
    pub(crate) key: KeyMaterial,
    /// Structural features beyond the base content.
    pub(crate) features: &'static [Feature],
}

impl KdbxCase {
    /// The password component of the case's key, if any.
    pub(crate) fn password(&self) -> Option<&'static str> {
        match self.key {
            KeyMaterial::PasswordOnly(password)
            | KeyMaterial::PasswordAndKeyfile { password, .. } => Some(password),
            KeyMaterial::KeyfileOnly { .. } => None,
        }
    }

    /// Every entry title the generated vault contains, sorted — what a
    /// projection of the built fixture must find. The generator knows
    /// because it wrote them.
    pub(crate) fn expected_titles(&self) -> Vec<&'static str> {
        let mut titles = vec!["Secondary Entry", "Test Entry"];
        for feature in self.features {
            match feature {
                Feature::Totp => titles.push("totp entry"),
                Feature::RecycleBin => titles.push("deleted entry"),
                Feature::DeletedObjectRecord | Feature::History => {}
            }
        }
        titles.sort_unstable();
        titles
    }
}

/// The cheap-but-real Argon2d parameters every generated case shares —
/// keepass-rs's own defaults for new databases, on the Argon2d variant.
/// Deliberately not hardened: these are test fixtures, and the KDF cost
/// is paid per build, per open, per save.
const fn argon2d() -> KdfConfig {
    KdfConfig::Argon2 {
        iterations: 50,
        memory: 1024 * 1024,
        parallelism: 4,
        version: rust_argon2::Version::Version10,
    }
}

/// The same parameters on the Argon2id variant — the modern default.
const fn argon2id() -> KdfConfig {
    KdfConfig::Argon2id {
        iterations: 50,
        memory: 1024 * 1024,
        parallelism: 4,
        version: rust_argon2::Version::Version13,
    }
}

/// The generated corpus: the config matrix the committed fixtures used to
/// carry, plus the structural features. Every case is KDBX 4.1 — the
/// on-disk format decision-3 fixes, and the only version keepass-rs
/// writes.
pub(crate) const GENERATED_CORPUS: &[KdbxCase] = &[
    KdbxCase {
        name: "argon2d-aes-gzip",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2d(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    KdbxCase {
        name: "argon2id-aes-gzip",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    KdbxCase {
        name: "argon2id-chacha20",
        cipher: OuterCipherConfig::ChaCha20,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    KdbxCase {
        name: "argon2id-twofish",
        cipher: OuterCipherConfig::Twofish,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    KdbxCase {
        name: "argon2d-chacha20",
        cipher: OuterCipherConfig::ChaCha20,
        kdf: argon2d(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    // The uncompressed path, folded into a cipher case rather than paying
    // for a case of its own.
    KdbxCase {
        name: "argon2d-twofish-uncompressed",
        cipher: OuterCipherConfig::Twofish,
        kdf: argon2d(),
        compression: CompressionConfig::None,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    KdbxCase {
        name: "aeskdf-aes",
        cipher: OuterCipherConfig::AES256,
        kdf: KdfConfig::Aes { rounds: 10_000 },
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[],
    },
    KdbxCase {
        name: "keyfile-only-raw32",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::KeyfileOnly {
            keyfile: KeyfileKind::Raw32,
        },
        features: &[],
    },
    KdbxCase {
        name: "password-and-keyfile-xmlv2",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordAndKeyfile {
            password: "demopass",
            keyfile: KeyfileKind::XmlV2,
        },
        features: &[],
    },
    KdbxCase {
        name: "totp-argon2id",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[Feature::Totp],
    },
    KdbxCase {
        name: "recycle-bin-argon2id",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[Feature::RecycleBin, Feature::DeletedObjectRecord],
    },
    KdbxCase {
        name: "history-argon2id",
        cipher: OuterCipherConfig::AES256,
        kdf: argon2id(),
        compression: CompressionConfig::GZip,
        key: KeyMaterial::PasswordOnly("demopass"),
        features: &[Feature::History],
    },
];

/// One case from the generated corpus, by name — the readable way tests
/// pin a specific case ("the argon2id one") without index arithmetic.
pub(crate) fn case(name: &str) -> &'static KdbxCase {
    GENERATED_CORPUS
        .iter()
        .find(|case| case.name == name)
        .unwrap_or_else(|| panic!("no generated corpus case named {name}"))
}

/// A generated fixture on disk, and the key material that opens it.
pub(crate) struct BuiltFixture {
    /// The vault file, inside a unique scratch directory.
    pub(crate) vault: PathBuf,
    /// The keyfile, when the case's key has one.
    pub(crate) keyfile: Option<PathBuf>,
    /// The password, when the case's key has one.
    pub(crate) password: Option<&'static str>,
}

/// The 32 raw bytes of the legacy keyfile form. A test constant, not a
/// secret: it exists to be a keyfile, not to protect anything.
const RAW32_KEY: [u8; 32] = [0x5A; 32];

/// The XML v2 keyfile document: 32 bytes of key material, base64-wrapped.
/// The payload is keepass-rs's own documented test vector.
const XMLV2_KEY: &str = "<KeyFile><Meta><Version>2.0</Version></Meta>\
<Key><Data>NXyYiJMHg3ls+eBmjbAjWec9lcOToJiofbhNiFMTJMw=</Data></Key></KeyFile>";

/// Build one case's vault (and keyfile, if any) into a unique scratch
/// directory. The fixture lands at `{scratch}/{name}.kdbx` next to its
/// keyfile, and opens with exactly the key material the case specifies.
pub(crate) fn build(case: &KdbxCase, tag: &str) -> BuiltFixture {
    let dir = scratch(tag);
    let vault = dir.join(format!("{}.kdbx", case.name));
    let keyfile = match case.key {
        KeyMaterial::PasswordOnly(_) => None,
        KeyMaterial::KeyfileOnly { keyfile } | KeyMaterial::PasswordAndKeyfile { keyfile, .. } => {
            let path = dir.join(match keyfile {
                KeyfileKind::Raw32 => "raw32.key",
                KeyfileKind::XmlV2 => "xmlv2.keyx",
            });
            let bytes = match keyfile {
                KeyfileKind::Raw32 => RAW32_KEY.to_vec(),
                KeyfileKind::XmlV2 => XMLV2_KEY.as_bytes().to_vec(),
            };
            std::fs::write(&path, bytes).expect("write keyfile");
            Some(path)
        }
    };

    let mut database = Database::new();
    database.config.version = DatabaseVersion::KDB4(1);
    database.config.outer_cipher_config = case.cipher.clone();
    database.config.compression_config = case.compression.clone();
    database.config.kdf_config = case.kdf.clone();
    database.meta.database_name = Some(case.name.to_string());

    // Base content: one group, two entries with the field shapes every
    // real vault has — protected secrets, multiline notes, a custom
    // protected field, tags.
    {
        let mut root = database.root_mut();
        root.name = "Root".to_string();
        let mut internet = root.add_group();
        internet.name = "Internet".to_string();

        let mut primary = internet.add_entry();
        primary.set_unprotected("Title", "Test Entry");
        primary.set_unprotected("UserName", "alice");
        primary.set_protected("Password", "correct horse battery staple");
        primary.set_unprotected("URL", "https://example.com/login");
        primary.set_unprotected("Notes", "line one\nline two");
        primary.set_protected("custom.field", "a custom secret");
        primary.tags = vec!["tag-one".into(), "tag-two".into()];

        let mut secondary = internet.add_entry();
        secondary.set_unprotected("Title", "Secondary Entry");
        secondary.set_unprotected("UserName", "bob");
        secondary.set_protected("Password", "another secret");
    }

    for feature in case.features {
        match feature {
            Feature::Totp => {
                let mut root = database.root_mut();
                let mut group = root.group_by_name_mut("Internet").expect("base group");
                let mut entry = group.add_entry();
                entry.set_unprotected("Title", "totp entry");
                entry.set_unprotected("UserName", "alice");
                entry.set_unprotected(
                    "otp",
                    "otpauth://totp/Castellan:alice?secret=JBSWY3DPEHPK3PXP&issuer=Castellan",
                );
            }
            Feature::RecycleBin => {
                // The group is built under the root borrow; the meta
                // wiring happens once that borrow has ended.
                let bin_uuid = {
                    let mut root = database.root_mut();
                    let mut bin = root.add_group();
                    bin.name = "Recycle Bin".to_string();
                    let uuid = bin.id().uuid();
                    let mut entry = bin.add_entry();
                    entry.set_unprotected("Title", "deleted entry");
                    entry.set_protected("Password", "a deleted secret");
                    uuid
                };
                database.meta.recyclebin_enabled = Some(true);
                database.meta.recyclebin_uuid = Some(bin_uuid);
            }
            Feature::DeletedObjectRecord => {
                // A purged object's uuid: remembered by the database, gone
                // from the tree. A fresh group uuid stands in for it.
                let purged = keepass::db::GroupId::new();
                database.deleted_objects.insert(purged.uuid(), None);
            }
            Feature::History => {
                let mut root = database.root_mut();
                let mut group = root.group_by_name_mut("Internet").expect("base group");
                let id = group
                    .as_ref()
                    .entries()
                    .find(|entry| entry.get_title() == Some("Test Entry"))
                    .expect("primary entry")
                    .id();
                let mut entry = group.entry_mut(id).expect("entry by its id");
                // The KeePass shape: the entry's own earlier state, same
                // uuid, snapshotted before the edit that follows.
                entry.set_protected("Password", "an older secret");
                let snapshot = keepass::db::Entry::clone(&entry);
                entry.set_protected("Password", "correct horse battery staple");
                entry
                    .history
                    .get_or_insert_with(Default::default)
                    .add_entry(snapshot);
            }
        }
    }

    let mut key = DatabaseKey::new();
    if let Some(password) = case.password() {
        key = key.with_password(password);
    }
    if let Some(keyfile) = &keyfile {
        let mut source = std::fs::File::open(keyfile).expect("reopen keyfile");
        key = key.with_keyfile(&mut source).expect("keyfile parses");
    }

    let mut target = std::fs::File::create(&vault).expect("create vault");
    database
        .save(&mut target, key)
        .unwrap_or_else(|error| panic!("{} must build: {error}", case.name));

    BuiltFixture {
        vault,
        keyfile,
        password: case.password(),
    }
}
