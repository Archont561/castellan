//! The save contract (task-8, decision-3): copy-aside before any write,
//! temp-then-rename, the original untouched on failure. These tests copy
//! corpus fixtures into unique temp directories and save through the vault
//! crate's front door — the only save path that exists.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use castellan_vault::{TEMP_SUFFIX, aside_name, open};
use rstest::{fixture, rstest};

/// Where the committed corpus lives.
fn corpus(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A unique scratch directory per call: tests run concurrently, and the
/// copy-aside contract is exactly the kind of thing a shared path would
/// make flaky.
fn scratch(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "castellan-save-tests-{}-{tag}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// A corpus fixture copied into a scratch directory, ready to be saved
/// over — the committed fixture itself is never written to.
#[fixture]
fn working_vault() -> (PathBuf, Vec<u8>) {
    let dir = scratch("working");
    let path = dir.join("vault.kdbx");
    std::fs::copy(corpus("test_db_kdbx4_with_password_argon2id.kdbx"), &path)
        .expect("copy fixture");
    let bytes = std::fs::read(&path).expect("read fixture");
    (path, bytes)
}

/// The temp sibling a save at `path` writes before renaming.
fn temp_sibling(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}.{TEMP_SUFFIX}",
        path.file_name().expect("file name").to_string_lossy()
    ))
}

#[rstest]
fn saving_writes_a_new_file_and_keeps_the_pre_save_bytes_aside(working_vault: (PathBuf, Vec<u8>)) {
    let (path, original) = working_vault;
    let mut handle = open(&path, Some("demopass"), None).expect("fixture opens");

    let outcome = handle.save().expect("save succeeds");

    // The copy-aside holds exactly the pre-save bytes: what the user would
    // lose if the new file turned out wrong.
    let aside = outcome.aside.expect("a prior file means a copy-aside");
    assert_eq!(std::fs::read(&aside).expect("aside exists"), original);
    // The aside is a timestamped sibling of the vault, named so a
    // directory listing reads as dates.
    assert_eq!(
        aside.parent(),
        path.parent(),
        "the copy-aside lives beside the vault"
    );
    // The vault path now holds a freshly written, openable database —
    // through the front door, the way the next session meets it.
    let reopened = open(&path, Some("demopass"), None).expect("the saved file reopens");
    assert_eq!(reopened.entries().len(), handle.entries().len());
    // And no temp file lingers: a successful save leaves two files, the
    // vault and its backup, nothing else.
    assert!(!temp_sibling(&path).exists(), "no temp file after success");
}

/// The no-aside branch through the public door: `open` requires the file
/// to exist, so the only way to reach a save with no previous file is the
/// vault being deleted (or moved) between unlock and save. Saving then
/// writes a fresh file and reports no copy-aside — there was nothing to
/// keep, and inventing one (e.g. from the in-memory image) would break
/// the rule that an aside is always a byte-exact prior file.
#[rstest]
fn a_save_with_no_previous_file_on_disk_writes_fresh_without_an_aside() {
    use keepass::{Database, DatabaseKey};
    let dir = scratch("first-save");
    let path = dir.join("brand-new.kdbx");
    let mut database = Database::new();
    database.root_mut().add_entry();
    let mut file = std::fs::File::create(&path).expect("create");
    database
        .save(&mut file, DatabaseKey::new().with_password("first"))
        .expect("seed file");

    let mut handle = open(&path, Some("first"), None).expect("seed opens");
    std::fs::remove_file(&path).expect("the vault vanishes before the save");
    let outcome = handle.save().expect("save succeeds");

    assert_eq!(
        outcome.aside, None,
        "no previous file on disk, no copy-aside"
    );
    assert!(path.exists(), "the save wrote the file back");
    open(&path, Some("first"), None).expect("the fresh file reopens");
}

/// A KDBX 3 vault opens (the read-compatibility promise) but is refused by
/// the save path *before touching the filesystem*: the on-disk format is
/// KDBX 4 (decision-3), and a refusal must not litter the directory with
/// backups of a file it never wrote.
#[rstest]
fn a_kdbx3_vault_is_read_compatible_but_save_refuses_cleanly() {
    let dir = scratch("kdbx3");
    let path = dir.join("old.kdbx");
    std::fs::copy(corpus("test_db_with_password.kdbx"), &path).expect("copy fixture");
    let original = std::fs::read(&path).expect("read fixture");

    let mut handle = open(&path, Some("demopass"), None).expect("KDBX 3 opens: read compat");
    assert!(!handle.entries().is_empty());

    let error = handle.save().expect_err("KDBX 3 must not be saved");
    assert!(
        matches!(error, castellan_vault::VaultError::Save(_)),
        "refusal is a save error, got {error:?}"
    );
    // Refused before any filesystem effect: byte-identical original, no
    // temp, no aside.
    assert_eq!(std::fs::read(&path).expect("still there"), original);
    assert!(!temp_sibling(&path).exists());
    assert_eq!(
        std::fs::read_dir(&dir).expect("dir").count(),
        1,
        "only the vault itself is in the directory"
    );
}

/// The proof of "copy-aside exists before any write" (AC-2): the write is
/// made to fail by blocking the temp path with a directory, and the
/// copy-aside exists anyway — the copy ran first. The original is
/// byte-identical because the temp-then-rename rule never let the writer
/// near the vault's own name.
#[rstest]
fn a_write_failure_leaves_the_original_untouched_and_proves_the_copy_ran_first(
    working_vault: (PathBuf, Vec<u8>),
) {
    let (path, original) = working_vault;
    let mut handle = open(&path, Some("demopass"), None).expect("fixture opens");

    // Block the temp sibling: File::create on a directory path fails.
    std::fs::create_dir_all(temp_sibling(&path)).expect("block the temp path");

    let error = handle
        .save()
        .expect_err("a blocked temp path fails the save");
    assert!(
        matches!(error, castellan_vault::VaultError::Io { .. }),
        "the failure is an io error, got {error:?}"
    );

    // The original is byte-identical: no partial write ever bore its name.
    assert_eq!(std::fs::read(&path).expect("still there"), original);
    // The copy-aside was made before the write was attempted — that is
    // the ordering this test exists to pin.
    let aside = std::fs::read_dir(path.parent().expect("dir"))
        .expect("dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|name| name.ends_with(".bak"));
    assert!(aside.is_some(), "the copy-aside ran before the write");
}

#[rstest]
#[case(0, "19700101T000000.000Z")]
#[case(951_782_400_000, "20000229T000000.000Z")] // leap year, non-divisible century
#[case(1_709_164_800_000, "20240229T000000.000Z")] // leap year
#[case(1_791_028_800_123, "20261003T120000.123Z")]
#[case(86_399_999, "19700101T235959.999Z")]
fn aside_names_carry_a_readable_utc_timestamp(
    #[case] millis_since_epoch: u64,
    #[case] expected: &str,
) {
    let now = SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(millis_since_epoch);
    assert_eq!(
        aside_name("vault.kdbx", now),
        format!("vault.kdbx.{expected}.bak")
    );
}
