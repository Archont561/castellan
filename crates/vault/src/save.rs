//! The save path: decision-3's copy-aside rule, as code.
//!
//! keepass-rs's writer is experimental and has a documented history of
//! dropping fields it does not parse, so saving is deliberately a ceremony:
//!
//! 1. **Refuse early.** A database that is not KDBX 4 is refused before
//!    touching the filesystem — the on-disk format is KDBX 4 and a KDBX 3
//!    file someone tries to save should not sprout backups.
//! 2. **Copy aside.** The current file is copied to a timestamped sibling
//!    (`vault.kdbx.20261003T114512.482Z.bak`) *before any write*. The copy
//!    is kept until the user prunes (default retention: 20 copies or 30
//!    days — the settings UI is task-15's; the policy lives in the docs
//!    here until then).
//! 3. **Write to a temp sibling, rename over.** The new database is
//!    written to `vault.kdbx.castellan-save.tmp` in the same directory
//!    (same filesystem, so the rename is atomic) and only then renamed
//!    onto the vault path. A failure anywhere before the rename leaves the
//!    original byte-identical — a partially written file is never visible
//!    under the vault's name.
//!
//! A failed save may leave a copy-aside behind (step 2 already happened).
//! That is safe by construction: the copy is a valid backup of the
//! pre-save file, not a corrupt half-state.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use keepass::config::DatabaseVersion;

use crate::{VaultError, VaultHandle};

/// What a successful save did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOutcome {
    /// The copy-aside this save made, if there was a previous file to
    /// copy. `None` means this was the vault's first save.
    pub aside: Option<PathBuf>,
}

/// The suffix every save's temporary sibling carries. Not hidden (no dot):
/// a leftover from a crashed save should be visible in a directory
/// listing, because it is exactly what "the vault looks wrong" debugging
/// wants to find.
pub const TEMP_SUFFIX: &str = "castellan-save.tmp";

/// The extension every copy-aside carries.
pub const ASIDE_EXTENSION: &str = "bak";

impl VaultHandle {
    /// Save the vault back to the path it was opened from, under the
    /// copy-aside rule (see the module docs). The original file is only
    /// ever replaced by an atomic rename; if this returns an error, the
    /// original is byte-identical to what it was.
    ///
    /// A KDBX 4.0 vault is saved as 4.1: keepass-rs's writer emits 4.1
    /// only, 4.1 is a backward-compatible extension of 4.0, and KeePassXC
    /// itself upgrades 4.0 files on save — the copy-aside keeps the 4.0
    /// original regardless. KDBX 3 stays refused (the on-disk format is
    /// KDBX 4, decision-3).
    ///
    /// The key retained from unlock is used to encrypt (and wipes with
    /// the handle); the path is the one [`crate::open`] was given.
    pub fn save(&mut self) -> Result<SaveOutcome, VaultError> {
        // Step 1: refuse non-KDBX4 before touching the filesystem, and
        // upgrade 4.0 to 4.1 so the writer (4.1-only) can proceed.
        match self.database.config.version {
            DatabaseVersion::KDB4(1) => {}
            DatabaseVersion::KDB4(_) => {
                self.database.config.version = DatabaseVersion::KDB4(1);
            }
            _ => {
                return Err(VaultError::Save(
                    keepass::db::DatabaseSaveError::UnsupportedVersion,
                ));
            }
        }

        // Step 2: copy the current file aside, timestamped, before any
        // write — the rule the whole module exists to enforce.
        let aside = if self.path.exists() {
            let aside = unique_aside_path(&self.path, SystemTime::now());
            std::fs::copy(&self.path, &aside).map_err(|source| VaultError::Io {
                path: aside.clone(),
                source,
            })?;
            Some(aside)
        } else {
            None
        };

        // Step 3: write to the temp sibling, then rename over the original.
        // Any failure below removes the temp file and leaves the original
        // untouched (the copy-aside from step 2 stays: a valid backup).
        let mut temp = self.path.as_os_str().to_os_string();
        temp.push(".");
        temp.push(TEMP_SUFFIX);
        let temp = PathBuf::from(temp);
        let write = (|| -> Result<(), VaultError> {
            let mut file = std::fs::File::create(&temp).map_err(|source| VaultError::Io {
                path: temp.clone(),
                source,
            })?;
            self.database
                .save(&mut file, self.key.clone())
                .map_err(VaultError::Save)?;
            // Flush the bytes and the metadata before the rename makes
            // them the vault: a rename of an unflushed buffer can survive
            // a crash as a zero-length file.
            file.sync_all().map_err(|source| VaultError::Io {
                path: temp.clone(),
                source,
            })?;
            Ok(())
        })();
        if let Err(error) = write {
            let _ = std::fs::remove_file(&temp);
            return Err(error);
        }

        std::fs::rename(&temp, &self.path).map_err(|source| VaultError::Io {
            path: self.path.clone(),
            source,
        })?;
        // The rename's directory entry should hit the disk too, where the
        // platform allows syncing a directory through std.
        #[cfg(unix)]
        if let Some(parent) = self.path.parent() {
            if let Ok(dir) = std::fs::File::open(parent) {
                let _ = dir.sync_all();
            }
        }

        Ok(SaveOutcome { aside })
    }
}

/// The copy-aside path for `source` at `now`: the file's own name, a dot,
/// the UTC timestamp, `.bak` — `vault.kdbx.20261003T114512.482Z.bak`.
/// Sortable by name because the timestamp is fixed-width, and readable in
/// a directory listing because it is a real date, not an epoch counter.
///
/// Public because the retention/prune policy (task-15's settings) prunes
/// by this naming contract.
#[must_use]
pub fn aside_name(file_name: &str, now: SystemTime) -> String {
    format!("{file_name}.{}.bak", utc_timestamp(now))
}

/// The first aside path that does not already exist: two saves inside one
/// millisecond get a numeric suffix instead of clobbering each other's
/// backup.
fn unique_aside_path(source: &Path, now: SystemTime) -> PathBuf {
    let file_name = source.file_name().map_or_else(
        || source.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let base = aside_name(&file_name, now);
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let mut candidate = parent.join(&base);
    let mut n = 1u32;
    while candidate.exists() {
        candidate = parent.join(format!("{base}.{n}.bak"));
        n += 1;
    }
    candidate
}

/// `YYYYMMDDTHHMMSS.mmmZ` in UTC, from a [`SystemTime`] — fixed-width,
/// lexicographically sortable, and readable without arithmetic.
fn utc_timestamp(now: SystemTime) -> String {
    let since_epoch = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = since_epoch.as_secs() as i64;
    let millis = since_epoch.subsec_millis();
    let days = secs.div_euclid(86_400);
    let seconds_of_day = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}{month:02}{day:02}T{:02}{:02}{:02}.{millis:03}Z",
        seconds_of_day / 3_600,
        (seconds_of_day % 3_600) / 60,
        seconds_of_day % 60,
    )
}

/// Days since the Unix epoch to a (year, month, day) civil date — Howard
/// Hinnant's `civil_from_days`, the standard minimal algorithm. The
/// inverse (days_from_civil) is the known-good reference the tests pin
/// this against.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
