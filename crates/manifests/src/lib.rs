//! The native-messaging manifest installer (task-10): detect the
//! installed browsers, write every host manifest in one pass, repair
//! them idempotently, and audit them for staleness.
//!
//! The single source of truth for the host identity and the extension
//! ids is `apps/extension/native-hosts/host.json`, which xtask turns
//! into [`generated::native_host`] — the installer never repeats those
//! strings, so a pinned store id lands everywhere at once.
//!
//! # The one-pass install
//!
//! [`Installer::install_all`] writes, for every browser whose
//! configuration directory exists, the manifest that browser's
//! native-messaging loader expects: a JSON file in the per-user
//! `NativeMessagingHosts` directory on Linux and macOS, a registry
//! value pointing at a common manifest file on Windows. Browsers that
//! are not installed are skipped and reported — writing
//! `~/.config/google-chrome` for a Chrome that does not exist would
//! leave confetti, not integration.
//!
//! # Repair is install
//!
//! The settings' Repair button runs the same [`Installer::install_all`]:
//! the write is idempotent (canonical content every time), and the
//! report distinguishes *created*, *updated* (with what changed) and
//! *already correct* — so repair both fixes and tells.
//!
//! # The audit
//!
//! [`Installer::audit`] reads each installed browser's manifest and
//! compares it against the expected one: a missing file, unparsable
//! JSON, a `path` that points somewhere the app no longer lives, an
//! allow-list missing one of our ids, or a foreign host's manifest
//! squatting the file name. Problems are
//! [`castellan_protocol::ManifestProblem`]s, which the task-9 panel
//! renders — the shell calls the audit when it builds the panel and
//! after every repair.
//!
//! # Platforms
//!
//! [`Platform`] is a value, not a `cfg`: the same build runs on every
//! desktop OS, and the tests exercise all three platforms' layouts from
//! Linux by rewriting the home directory. The Windows *registry write*
//! is the one stub — it lands with the Windows CI lane, exactly like
//! the IPC server's named-pipe listener — but the layout table (which
//! key, which value, which file) is complete and tested here.

mod install;
mod layout;

pub use install::{BrowserOutcome, Change, InstallReport, Installer};
pub use layout::{Browser, Family, ManifestLocation, Platform, manifest_location};

/// The host identity the installer writes: the same facts
/// `apps/extension/native-hosts/host.json` states, from the generated
/// constants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostIdentity {
    /// The native host's registered name (the file name and the registry
    /// value name).
    pub name: String,
    /// The one-line description every manifest carries.
    pub description: String,
    /// Extension ids allowed to connect, chromium family: the dev id and
    /// the ids pinned at first store submission.
    pub chromium_extension_ids: Vec<String>,
    /// Extension ids allowed to connect, gecko family.
    pub firefox_extension_ids: Vec<String>,
}

impl HostIdentity {
    /// The identity from the generated constants — production wiring.
    ///
    /// The store-id placeholders ride along as-is: they are replaced in
    /// `host.json` the day the ids are assigned, and the next `bun run
    /// codegen` propagates them to every manifest the installer writes.
    #[must_use]
    pub fn generated() -> Self {
        Self {
            name: generated::native_host::HOST_NAME.to_string(),
            description: generated::native_host::HOST_DESCRIPTION.to_string(),
            chromium_extension_ids: generated::native_host::CHROMIUM_EXTENSION_IDS
                .iter()
                .map(ToString::to_string)
                .collect(),
            firefox_extension_ids: generated::native_host::FIREFOX_EXTENSION_IDS
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }

    /// The allow-list a browser family sees: origins for chromium, bare
    /// ids for gecko — the one difference between the two manifest
    /// shapes that is not layout.
    #[must_use]
    pub fn allow_list(&self, family: Family) -> Vec<String> {
        match family {
            Family::Chromium => self
                .chromium_extension_ids
                .iter()
                .map(|id| format!("chrome-extension://{id}/"))
                .collect(),
            Family::Gecko => self.firefox_extension_ids.clone(),
        }
    }
}

pub mod generated {
    //! The committed generated directory (xtask-owned, like every other
    //! generated dir in the workspace).

    /// The host identity from `apps/extension/native-hosts/host.json`.
    pub mod native_host;
}
