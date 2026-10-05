//! TOTP seed access and batch import, through the session's public API:
//! the same calls the dispatcher makes for `get_totp` and
//! `import_otp_accounts`.

use castellan_protocol::Event;
use castellan_vault::{TotpImport, VaultError, VaultSession};
use keepass::{Database, DatabaseKey};
use rstest::{fixture, rstest};
use std::sync::{Arc, Mutex};

const PASSWORD: &str = "fixture password, never a real one";
const SEED: &str = "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP&issuer=GitHub";

/// A saved vault on disk with one TOTP entry and one plain entry, plus
/// the directory guard that cleans it (and its copy-asides) up.
struct DiskVault {
    directory: tempdir::Guard,
    path: std::path::PathBuf,
}

/// The tiniest possible scoped temp dir — std-only, no tempfile crate
/// in the vendor set, and `Drop` cleans up copy-asides too.
mod tempdir {
    pub(crate) struct Guard(pub(crate) std::path::PathBuf);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    pub(crate) fn scoped(name: &str) -> Guard {
        let path = std::env::temp_dir().join(format!(
            "castellan-totp-import-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Guard(path)
    }
}

#[fixture]
fn disk_vault() -> DiskVault {
    let directory = tempdir::scoped("vault");
    let path = directory.0.join("vault.kdbx");
    let mut database = Database::new();
    {
        let mut root = database.root_mut();
        let mut with_totp = root.add_entry();
        with_totp.set_unprotected("Title", "GitHub");
        with_totp.set_protected("otp", SEED);
        let mut plain = root.add_entry();
        plain.set_unprotected("Title", "No 2FA here");
    }
    let mut file = std::fs::File::create(&path).unwrap();
    database
        .save(&mut file, DatabaseKey::new().with_password(PASSWORD))
        .unwrap();
    DiskVault { directory, path }
}

fn unlocked(disk_vault: &DiskVault) -> VaultSession {
    let session = VaultSession::new();
    session
        .unlock(&disk_vault.path, Some(PASSWORD), None)
        .unwrap();
    session
}

#[rstest]
fn the_stored_seed_comes_back_verbatim(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    let entries = session.entries().unwrap();
    let with_totp = entries.iter().find(|entry| entry.has_totp).unwrap();
    assert_eq!(session.totp_uri(&with_totp.id).unwrap(), SEED);
}

#[rstest]
fn an_entry_without_a_seed_says_so(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    let entries = session.entries().unwrap();
    let plain = entries.iter().find(|entry| !entry.has_totp).unwrap();
    assert!(matches!(
        session.totp_uri(&plain.id),
        Err(VaultError::NoTotpSeed { .. })
    ));
}

#[rstest]
fn a_missing_entry_is_no_such_entry(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    assert!(matches!(
        session.totp_uri("00000000-0000-0000-0000-000000000000"),
        Err(VaultError::NoSuchEntry { .. })
    ));
}

#[rstest]
fn a_locked_session_refuses_before_existence(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    let id = session.entries().unwrap()[0].id.clone();
    session.lock(castellan_vault::LockReason::User);
    // Locked wins over "no such entry" too: a locked vault must not
    // confirm which ids exist.
    assert!(matches!(session.totp_uri(&id), Err(VaultError::Locked)));
    assert!(matches!(
        session.totp_uri("not-an-id"),
        Err(VaultError::Locked)
    ));
}

#[rstest]
fn an_imported_batch_survives_relock_and_reopen(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = {
        let heard = Arc::clone(&heard);
        Arc::new(move |event: &Event| heard.lock().unwrap().push(event.clone()))
    };
    session.subscribe(sink);

    let ids = session
        .import_totp_entries(&[
            TotpImport {
                title: "Example".into(),
                username: Some("alice@example.org".into()),
                otpauth: "otpauth://totp/Example:alice@example.org?secret=JBSWY3DPEHPK3PXP".into(),
            },
            TotpImport {
                title: "Bank".into(),
                username: None,
                otpauth: "otpauth://totp/Bank:bob?secret=JBSWY3DPEHPK3PXP&digits=8".into(),
            },
        ])
        .unwrap();
    assert_eq!(ids.len(), 2);

    // Every new entry was announced, by its real id.
    let events = heard.lock().unwrap();
    for id in &ids {
        assert!(events.contains(&Event::EntryChanged { id: id.clone() }));
    }
    drop(events);

    // The import saved under the copy-aside rule: the original bytes
    // are a sibling .bak now.
    let asides = std::fs::read_dir(&disk_vault.directory.0)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("bak")
        })
        .count();
    assert_eq!(asides, 1);

    // Reopen from disk: the batch is really in the file, seeds intact.
    let reopened = VaultSession::new();
    reopened
        .unlock(&disk_vault.path, Some(PASSWORD), None)
        .unwrap();
    let entries = reopened.entries().unwrap();
    assert_eq!(entries.len(), 4);
    let bank = entries.iter().find(|entry| entry.title == "Bank").unwrap();
    assert!(bank.has_totp);
    assert_eq!(
        reopened.totp_uri(&bank.id).unwrap(),
        "otpauth://totp/Bank:bob?secret=JBSWY3DPEHPK3PXP&digits=8"
    );
    let example = entries
        .iter()
        .find(|entry| entry.title == "Example")
        .unwrap();
    assert_eq!(example.username.as_deref(), Some("alice@example.org"));
}

#[rstest]
fn a_locked_session_refuses_imports(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    session.lock(castellan_vault::LockReason::User);
    assert!(matches!(
        session.import_totp_entries(&[TotpImport {
            title: "X".into(),
            username: None,
            otpauth: "otpauth://totp/X:x?secret=JBSWY3DPEHPK3PXP".into(),
        }]),
        Err(VaultError::Locked)
    ));
}

#[rstest]
fn an_empty_batch_is_a_no_op(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    assert_eq!(
        session.import_totp_entries(&[]).unwrap(),
        Vec::<String>::new()
    );
    // No save happened: no copy-aside appeared.
    let asides = std::fs::read_dir(&disk_vault.directory.0)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("bak")
        })
        .count();
    assert_eq!(asides, 0);
}

#[rstest]
fn a_failed_save_rolls_the_batch_back(disk_vault: DiskVault) {
    let session = unlocked(&disk_vault);
    // Make the save path unwritable: a directory squatting on the temp
    // sibling's name makes `File::create` fail before the rename, which
    // is exactly the failure the rollback must cover.
    let temp_sibling = disk_vault
        .directory
        .0
        .join(format!("vault.kdbx.{}", castellan_vault::TEMP_SUFFIX));
    std::fs::create_dir(&temp_sibling).unwrap();

    let result = session.import_totp_entries(&[TotpImport {
        title: "Doomed".into(),
        username: None,
        otpauth: "otpauth://totp/Doomed:x?secret=JBSWY3DPEHPK3PXP".into(),
    }]);
    assert!(result.is_err());

    // Memory agrees with disk: the entry is not in the session either.
    let entries = session.entries().unwrap();
    assert!(!entries.iter().any(|entry| entry.title == "Doomed"));
}
