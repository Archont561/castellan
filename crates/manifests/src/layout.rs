//! Where each browser's native-messaging manifest lives, per platform —
//! one table, no cfg, so the same build answers for every desktop OS and
//! the tests can exercise all three layouts from any of them.
//!
//! Sources: Chrome's native-messaging documentation (developer.chrome.com
//! /docs/extensions/develop/concepts/native-messaging) and Mozilla's
//! (developer.mozilla.org Native-Messaging), cross-checked against the
//! browser families' directory conventions. The per-user locations are
//! the ones a single-user install may write without elevation; the
//! system-wide locations are deliberately not touched by this installer.

use std::path::PathBuf;

use castellan_protocol::FaceKind;

/// A browser with a native-messaging loader we install for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Browser {
    /// Google Chrome (and Chromium, which shares the layout).
    Chrome,
    /// Microsoft Edge.
    Edge,
    /// Brave.
    Brave,
    /// Vivaldi.
    Vivaldi,
    /// Firefox (and derivatives that share its native-messaging layout).
    Firefox,
}

impl Browser {
    /// Every supported browser, in a stable order (the report's order).
    pub const ALL: [Browser; 5] = [
        Browser::Chrome,
        Browser::Edge,
        Browser::Brave,
        Browser::Vivaldi,
        Browser::Firefox,
    ];

    /// Which manifest shape the browser reads.
    #[must_use]
    pub fn family(self) -> Family {
        match self {
            Browser::Chrome | Browser::Edge | Browser::Brave | Browser::Vivaldi => Family::Chromium,
            Browser::Firefox => Family::Gecko,
        }
    }

    /// The panel's label for this browser.
    #[must_use]
    pub fn face(self) -> FaceKind {
        match self {
            Browser::Chrome => FaceKind::Chrome,
            Browser::Edge => FaceKind::Edge,
            Browser::Brave => FaceKind::Brave,
            Browser::Vivaldi => FaceKind::Vivaldi,
            Browser::Firefox => FaceKind::Firefox,
        }
    }

    /// The human name, for reports.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Browser::Chrome => "Chrome",
            Browser::Edge => "Edge",
            Browser::Brave => "Brave",
            Browser::Vivaldi => "Vivaldi",
            Browser::Firefox => "Firefox",
        }
    }

    /// Where the browser keeps its per-user configuration — the directory
    /// whose existence means "installed, with a user profile that could
    /// talk to a host".
    #[must_use]
    pub fn profile_dir(self, platform: Platform, home: &std::path::Path) -> PathBuf {
        match platform {
            Platform::Linux => match self {
                Browser::Chrome => home.join(".config/google-chrome"),
                Browser::Edge => home.join(".config/microsoft-edge"),
                Browser::Brave => home.join(".config/BraveSoftware/Brave-Browser"),
                Browser::Vivaldi => home.join(".config/vivaldi"),
                Browser::Firefox => home.join(".mozilla"),
            },
            Platform::MacOs => match self {
                Browser::Chrome => home.join("Library/Application Support/Google/Chrome"),
                Browser::Edge => home.join("Library/Application Support/Microsoft Edge"),
                Browser::Brave => {
                    home.join("Library/Application Support/BraveSoftware/Brave-Browser")
                }
                Browser::Vivaldi => home.join("Library/Application Support/Vivaldi"),
                Browser::Firefox => home.join("Library/Application Support/Firefox"),
            },
            // Windows: profiles live under %LOCALAPPDATA%\<Vendor>\<Browser>,
            // which is where the browsers also look up nothing — the
            // manifest is a registry value, so installation detection
            // uses the registry layout, not the profile.
            Platform::Windows => match self {
                Browser::Chrome => home.join("AppData/Local/Google/Chrome"),
                Browser::Edge => home.join("AppData/Local/Microsoft/Edge"),
                Browser::Brave => home.join("AppData/Local/BraveSoftware/Brave-Browser"),
                Browser::Vivaldi => home.join("AppData/Local/Vivaldi"),
                Browser::Firefox => home.join("AppData/Local/Mozilla/Firefox"),
            },
        }
    }
}

/// The two manifest shapes a browser family reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// Chrome-derived: `allowed_origins` of `chrome-extension://<id>/`.
    Chromium,
    /// Firefox-derived: `allowed_extensions` of bare ids.
    Gecko,
}

/// The desktop platforms the installer knows. A value, not a `cfg`:
/// the layouts are data, the tests run all three from any OS, and the
/// one platform-specific *action* (the Windows registry write) is stubbed
/// at the write, not hidden in the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux: XDG config directories.
    Linux,
    /// macOS: `~/Library/Application Support`.
    MacOs,
    /// Windows: HKCU registry values pointing at a manifest file.
    Windows,
}

impl Platform {
    /// The platform this build runs on.
    #[must_use]
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(target_os = "macos")]
        {
            Platform::MacOs
        }
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Platform::Linux
        }
    }
}

/// Where a browser's manifest lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestLocation {
    /// The manifest is this JSON file (Linux, macOS).
    File(PathBuf),
    /// The manifest is a registry value at this key (under HKCU),
    /// pointing at this JSON file (Windows). The file is shared by every
    /// browser; the key is per-browser.
    Registry {
        /// The registry key, relative to HKEY_CURRENT_USER.
        key: String,
        /// The manifest file the value points at.
        file: PathBuf,
    },
}

impl ManifestLocation {
    /// The JSON file to read or write, whichever shape this location is.
    #[must_use]
    pub fn file(&self) -> &std::path::Path {
        match self {
            ManifestLocation::File(path) => path,
            ManifestLocation::Registry { file, .. } => file,
        }
    }
}

/// Where `browser` looks for the host named `host_name`, under the given
/// home directory (the tests' fake home; production passes the real one).
#[must_use]
pub fn manifest_location(
    browser: Browser,
    platform: Platform,
    home: &std::path::Path,
    host_name: &str,
) -> ManifestLocation {
    match platform {
        Platform::Linux => {
            // Mozilla named its directory in the lowercase (its docs and
            // firefox source agree); the chromium family uses the
            // CamelCase one. Both are per-user.
            let dir = match browser {
                Browser::Chrome => ".config/google-chrome/NativeMessagingHosts",
                Browser::Edge => ".config/microsoft-edge/NativeMessagingHosts",
                Browser::Brave => ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts",
                Browser::Vivaldi => ".config/vivaldi/NativeMessagingHosts",
                Browser::Firefox => ".mozilla/native-messaging-hosts",
            };
            ManifestLocation::File(home.join(dir).join(format!("{host_name}.json")))
        }
        Platform::MacOs => {
            let dir = match browser {
                Browser::Chrome => "Library/Application Support/Google/Chrome",
                Browser::Edge => "Library/Application Support/Microsoft Edge",
                Browser::Brave => "Library/Application Support/BraveSoftware/Brave-Browser",
                Browser::Vivaldi => "Library/Application Support/Vivaldi",
                Browser::Firefox => "Library/Application Support/Mozilla",
            };
            ManifestLocation::File(
                home.join(dir)
                    .join("NativeMessagingHosts")
                    .join(format!("{host_name}.json")),
            )
        }
        Platform::Windows => {
            // Firefox on Windows also reads the registry (Mozilla chose
            // parity with its own loader); the chromium family reads its
            // vendor key. The manifest file itself lives in one shared
            // place, because five identical copies would be five chances
            // to be stale.
            let key = match browser {
                Browser::Chrome => r"Software\Google\Chrome",
                Browser::Edge => r"Software\Microsoft\Edge",
                Browser::Brave => r"Software\BraveSoftware\Brave-Browser",
                Browser::Vivaldi => r"Software\Vivaldi",
                Browser::Firefox => r"Software\Mozilla",
            };
            ManifestLocation::Registry {
                key: format!(r"{key}\NativeMessagingHosts\{host_name}"),
                file: home
                    .join("AppData/Local/Castellan")
                    .join(format!("{host_name}.json")),
            }
        }
    }
}
