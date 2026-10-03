//! The one-pass installer: detect, write, report. And the audit that
//! finds staleness before a browser silently stops talking to the app.

use std::path::{Path, PathBuf};

use castellan_protocol::{ManifestProblem, ManifestProblemKind};

use crate::HostIdentity;
use crate::layout::{Browser, Family, ManifestLocation, Platform, manifest_location};

/// What happened to one browser in an install pass. The report is the
/// settings UI's transcript — every browser appears exactly once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserOutcome {
    /// No manifest existed; one was written.
    Written {
        /// Which browser.
        browser: Browser,
        /// The manifest file written.
        path: PathBuf,
    },
    /// A manifest existed but differed; it was rewritten, and this is what
    /// changed.
    Repaired {
        /// Which browser.
        browser: Browser,
        /// The manifest file rewritten.
        path: PathBuf,
        /// What the repair changed.
        changes: Vec<Change>,
    },
    /// The manifest was already canonical; nothing was written.
    AlreadyCorrect {
        /// Which browser.
        browser: Browser,
        /// The manifest file left untouched.
        path: PathBuf,
    },
    /// The browser is not installed (no profile directory); no manifest
    /// was written for it.
    SkippedNotInstalled {
        /// Which browser.
        browser: Browser,
    },
    /// The step needs a platform capability this build does not carry
    /// (the Windows registry write, until its CI lane).
    Unsupported {
        /// Which browser.
        browser: Browser,
        /// Why, in one sentence for the settings UI.
        why: String,
    },
    /// The write was attempted and failed.
    Failed {
        /// Which browser.
        browser: Browser,
        /// The I/O failure, for the settings UI.
        why: String,
    },
}

/// What a repair changed in one manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The manifest did not exist.
    Created,
    /// The host path moved with the app.
    PathUpdated {
        /// Where the manifest pointed.
        from: String,
        /// Where the app lives now.
        to: String,
    },
    /// The allow-list grew (or shrank) to match the current ids.
    AllowListUpdated {
        /// The ids the list was missing.
        missing: Vec<String>,
    },
    /// The file was not Castellan's manifest, or was unparsable; it was
    /// replaced wholesale.
    Replaced {
        /// Why, in one sentence.
        why: String,
    },
}

/// The transcript of one install pass — every supported browser, exactly
/// once, in [`Browser::ALL`] order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    /// The platform the pass ran for.
    pub platform: Platform,
    /// One outcome per browser, in stable order.
    pub outcomes: Vec<BrowserOutcome>,
}

impl InstallReport {
    /// The browsers a repair actually touched (written or repaired).
    #[must_use]
    pub fn fixed(&self) -> Vec<&Browser> {
        self.outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    BrowserOutcome::Written { .. } | BrowserOutcome::Repaired { .. }
                )
            })
            .map(|outcome| outcome.browser())
            .collect()
    }
}

impl BrowserOutcome {
    /// Which browser this outcome is about.
    #[must_use]
    pub fn browser(&self) -> &Browser {
        match self {
            BrowserOutcome::Written { browser, .. }
            | BrowserOutcome::Repaired { browser, .. }
            | BrowserOutcome::AlreadyCorrect { browser, .. }
            | BrowserOutcome::SkippedNotInstalled { browser }
            | BrowserOutcome::Unsupported { browser, .. }
            | BrowserOutcome::Failed { browser, .. } => browser,
        }
    }
}

/// The installer. Construct with the generated [`HostIdentity`], the app
/// binary's absolute path (what the manifests' `path` must point at), a
/// home directory to work under, and the platform to install for.
#[derive(Debug, Clone)]
pub struct Installer {
    identity: HostIdentity,
    binary_path: PathBuf,
    home: PathBuf,
    platform: Platform,
}

impl Installer {
    /// The installer for `platform`, working under `home` (the real home
    /// directory in production, a scratch directory in tests), writing
    /// manifests that point at `binary_path`.
    #[must_use]
    pub fn new(
        identity: HostIdentity,
        binary_path: PathBuf,
        home: PathBuf,
        platform: Platform,
    ) -> Self {
        Self {
            identity,
            binary_path,
            home,
            platform,
        }
    }

    /// The browsers this machine has installed — a browser is installed
    /// when its per-user configuration directory exists under the home.
    #[must_use]
    pub fn detect_installed(&self) -> Vec<Browser> {
        Browser::ALL
            .into_iter()
            .filter(|browser| browser.profile_dir(self.platform, &self.home).exists())
            .collect()
    }

    /// The one-pass install, which is also the repair: for every
    /// *installed* browser, write the canonical manifest, and report
    /// exactly what that did. Idempotent by construction — the second
    /// pass reports [`BrowserOutcome::AlreadyCorrect`] everywhere.
    ///
    /// On Windows the registry write lands with the Windows CI lane:
    /// every browser reports [`BrowserOutcome::Unsupported`] and nothing
    /// is touched, because a manifest file without its registry value is
    /// integration that only looks installed.
    pub fn install_all(&self) -> InstallReport {
        let installed: Vec<Browser> = self.detect_installed();
        let mut outcomes = Vec::new();
        for browser in Browser::ALL {
            let installed = installed.contains(&browser);
            outcomes.push(self.install_one(browser, installed));
        }
        InstallReport {
            platform: self.platform,
            outcomes,
        }
    }

    fn install_one(&self, browser: Browser, installed: bool) -> BrowserOutcome {
        if !installed {
            return BrowserOutcome::SkippedNotInstalled { browser };
        }
        let location = manifest_location(browser, self.platform, &self.home, &self.identity.name);
        if matches!(location, ManifestLocation::Registry { .. }) {
            // The Windows registry write is the CI lane's, not a guess:
            // report it rather than half-installing.
            return BrowserOutcome::Unsupported {
                browser,
                why: "the Windows registry write lands with the Windows CI lane (task-10 notes)"
                    .to_string(),
            };
        }
        let path = location.file().to_path_buf();
        let canonical = self.manifest_content(browser.family());
        match std::fs::read_to_string(&path) {
            Ok(current) if current == canonical => BrowserOutcome::AlreadyCorrect { browser, path },
            Ok(current) => match self.write(&path, &canonical) {
                Ok(()) => {
                    let changes = self.describe_change(&current, &path, browser.family());
                    BrowserOutcome::Repaired {
                        browser,
                        path,
                        changes,
                    }
                }
                Err(cause) => BrowserOutcome::Failed {
                    browser,
                    why: cause,
                },
            },
            Err(_) => match self.write(&path, &canonical) {
                // A missing file and an unreadable one both land here;
                // the repair report says which.
                Ok(()) => BrowserOutcome::Written { browser, path },
                Err(cause) => BrowserOutcome::Failed {
                    browser,
                    why: cause,
                },
            },
        }
    }

    /// The staleness audit: for every installed browser, read the
    /// manifest the loader would read and compare it against the one
    /// this app would write. Problems are the panel's rows.
    ///
    /// On Windows the audit stays silent (the registry, not the file, is
    /// the source of truth there, and reading it needs the same CI lane
    /// as writing it) — an empty list must never claim more than was
    /// checked.
    #[must_use]
    pub fn audit(&self) -> Vec<ManifestProblem> {
        if self.platform == Platform::Windows {
            return Vec::new();
        }
        let mut problems = Vec::new();
        for browser in self.detect_installed() {
            problems.extend(self.audit_one(browser));
        }
        problems
    }

    fn audit_one(&self, browser: Browser) -> Vec<ManifestProblem> {
        let face = browser.face();
        let location = manifest_location(browser, self.platform, &self.home, &self.identity.name);
        let path = location.file();
        let Ok(raw) = std::fs::read_to_string(path) else {
            return vec![ManifestProblem {
                browser: face,
                kind: ManifestProblemKind::Missing,
                detail: format!("no manifest at {}", path.display()),
            }];
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return vec![ManifestProblem {
                browser: face,
                kind: ManifestProblemKind::Unreadable,
                detail: format!("{} is not valid JSON", path.display()),
            }];
        };
        let name = parsed.get("name").and_then(serde_json::Value::as_str);
        if name != Some(self.identity.name.as_str()) {
            return vec![ManifestProblem {
                browser: face,
                kind: ManifestProblemKind::Foreign,
                detail: format!(
                    "{} carries {}'s manifest",
                    path.display(),
                    name.unwrap_or("(no name)")
                ),
            }];
        }
        let mut problems = Vec::new();
        let manifest_path = parsed.get("path").and_then(serde_json::Value::as_str);
        if manifest_path != Some(self.binary_path.to_string_lossy().as_ref()) {
            problems.push(ManifestProblem {
                browser: face,
                kind: ManifestProblemKind::StalePath,
                detail: format!(
                    "points at {}, the app now lives at {}",
                    manifest_path.unwrap_or("(no path)"),
                    self.binary_path.display()
                ),
            });
        }
        let allow_list = self.identity.allow_list(browser.family());
        let field = match browser.family() {
            crate::Family::Chromium => "allowed_origins",
            crate::Family::Gecko => "allowed_extensions",
        };
        let present: Vec<String> = parsed
            .get(field)
            .and_then(serde_json::Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let missing: Vec<String> = allow_list
            .iter()
            .filter(|allowed| !present.contains(allowed))
            .cloned()
            .collect();
        if !missing.is_empty() {
            problems.push(ManifestProblem {
                browser: face,
                kind: ManifestProblemKind::MissingId,
                detail: format!("missing from {field}: {}", missing.join(", ")),
            });
        }
        problems
    }

    /// The canonical manifest bytes for a family: the same shape xtask
    /// templates into `apps/extension/native-hosts/generated/`, with the
    /// `{path}` placeholder replaced by this app's binary. Public because
    /// "show me the manifest you would write" is a settings question
    /// worth answering without installing anything.
    #[must_use]
    pub fn manifest_content(&self, family: Family) -> String {
        let mut manifest = serde_json::json!({
            "name": self.identity.name,
            "description": self.identity.description,
            "path": self.binary_path.to_string_lossy(),
            "type": "stdio",
        });
        let field = match family {
            crate::Family::Chromium => "allowed_origins",
            crate::Family::Gecko => "allowed_extensions",
        };
        manifest[field] = serde_json::json!(self.identity.allow_list(family));
        let mut rendered = serde_json::to_string_pretty(&manifest)
            .expect("manifest values are always serializable");
        rendered.push('\n');
        rendered
    }

    fn write(&self, path: &Path, contents: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|cause| format!("creating {}: {cause}", parent.display()))?;
        }
        std::fs::write(path, contents)
            .map_err(|cause| format!("writing {}: {cause}", path.display()))
    }

    /// Describe, from the bytes that were there before the rewrite, what
    /// the repair changed — the transcript the settings UI shows. The old
    /// bytes are already replaced when this runs; everything it reports
    /// comes from them, everything it promises comes from the installer.
    fn describe_change(&self, previous: &str, path: &Path, family: Family) -> Vec<Change> {
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(previous) else {
            return vec![Change::Replaced {
                why: format!("{} was not valid JSON", path.display()),
            }];
        };
        let name = parsed.get("name").and_then(serde_json::Value::as_str);
        if name != Some(self.identity.name.as_str()) {
            return vec![Change::Replaced {
                why: format!(
                    "the file carried {} manifest",
                    name.unwrap_or("an unnamed host's")
                ),
            }];
        }
        let mut changes = Vec::new();
        let old_path = parsed
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if old_path != self.binary_path.to_string_lossy() {
            changes.push(Change::PathUpdated {
                from: old_path.to_string(),
                to: self.binary_path.to_string_lossy().into_owned(),
            });
        }
        let field = match family {
            Family::Chromium => "allowed_origins",
            Family::Gecko => "allowed_extensions",
        };
        let present: Vec<String> = parsed
            .get(field)
            .and_then(serde_json::Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let missing: Vec<String> = self
            .identity
            .allow_list(family)
            .into_iter()
            .filter(|allowed| !present.contains(allowed))
            .collect();
        if !missing.is_empty() {
            changes.push(Change::AllowListUpdated { missing });
        }
        if changes.is_empty() {
            // The bytes differed but the load-bearing fields matched:
            // description or key order. Honest, if rare.
            changes.push(Change::Replaced {
                why: "the manifest's metadata changed".to_string(),
            });
        }
        changes
    }
}
