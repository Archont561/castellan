//! Entry projection: what the app faces see when they ask for entries.
//!
//! Integration tests over the crate's own front door — the fixture below
//! is a real KDBX round trip (build in memory, save, `open`), so `open`
//! itself stays covered, which the in-memory fixture this replaced never
//! did.

use castellan_vault::VaultHandle;
use keepass::{Database, DatabaseKey};
use rstest::{fixture, rstest};

/// An in-memory vault with one fully-shaped entry, written to a KDBX file
/// and opened back through the public API — the pytest-fixture pattern,
/// via rstest. Fresh instance per test, so a test mutating its vault
/// cannot leak state.
#[fixture]
fn sample_vault() -> VaultHandle {
    let mut database = Database::new();
    {
        let mut root = database.root_mut();
        let mut entry = root.add_entry();
        entry.set_unprotected("Title", "GitHub");
        entry.set_unprotected("UserName", "octocat");
        entry.set_unprotected("URL", "https://github.com");
        entry.set_protected(
            "otp",
            "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP",
        );
    }
    let path =
        std::env::temp_dir().join(format!("castellan-vault-tests-{}.kdbx", std::process::id(),));
    let mut file = std::fs::File::create(&path).unwrap();
    database
        .save(
            &mut file,
            DatabaseKey::new().with_password("fixture password, never a real one"),
        )
        .unwrap();
    drop(file);
    let vault = castellan_vault::open(&path, "fixture password, never a real one", None)
        .expect("a database this crate just saved must open back");
    let _ = std::fs::remove_file(&path);
    vault
}

#[rstest]
fn projects_entries_to_the_protocol_shape(sample_vault: VaultHandle) {
    let entries = sample_vault.entries();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.title, "GitHub");
    assert_eq!(entry.username.as_deref(), Some("octocat"));
    assert_eq!(entry.url.as_deref(), Some("https://github.com"));
    assert!(entry.has_totp);
    assert!(!entry.has_passkey);
}
