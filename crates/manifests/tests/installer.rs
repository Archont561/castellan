//! The manifest installer suite: every rule task-10 states as behavior —
//! one-pass install, idempotent repair with a transcript, staleness
//! audit — driven against a fake home directory, one platform layout at
//! a time. `Platform` is a value, so Linux runs the macOS and Windows
//! tables too.

use std::path::{Path, PathBuf};

use castellan_manifests::{Browser, Family, HostIdentity, Installer, Platform, manifest_location};
use castellan_protocol::{FaceKind, ManifestProblemKind};
use rstest::rstest;

fn scratch(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "castellan-manifests-{}-{tag}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// A home with only the named browsers "installed".
fn home_with(platform: Platform, browsers: &[Browser]) -> PathBuf {
    let home = scratch("home");
    for browser in browsers {
        std::fs::create_dir_all(browser.profile_dir(platform, &home)).expect("profile dir");
    }
    home
}

fn installer(home: &Path, platform: Platform) -> Installer {
    Installer::new(
        HostIdentity::generated(),
        PathBuf::from("/opt/castellan/castellan"),
        home.to_path_buf(),
        platform,
    )
}

/// A manifest with one field overridden, for staleness fixtures.
fn manifest_with(
    identity: &HostIdentity,
    family: Family,
    path: &str,
    allowed: &[String],
) -> String {
    let field = match family {
        Family::Chromium => "allowed_origins",
        Family::Gecko => "allowed_extensions",
    };
    let mut manifest = serde_json::json!({
        "name": identity.name,
        "description": identity.description,
        "path": path,
        "type": "stdio",
    });
    manifest[field] = serde_json::json!(allowed);
    let mut rendered = serde_json::to_string_pretty(&manifest).expect("serializable");
    rendered.push('\n');
    rendered
}

// ── Layout ───────────────────────────────────────────────────────────────────

#[rstest]
#[case::chrome_linux(
    Browser::Chrome,
    Platform::Linux,
    ".config/google-chrome/NativeMessagingHosts"
)]
#[case::edge_linux(
    Browser::Edge,
    Platform::Linux,
    ".config/microsoft-edge/NativeMessagingHosts"
)]
#[case::brave_linux(
    Browser::Brave,
    Platform::Linux,
    ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
)]
#[case::vivaldi_linux(
    Browser::Vivaldi,
    Platform::Linux,
    ".config/vivaldi/NativeMessagingHosts"
)]
#[case::firefox_linux(Browser::Firefox, Platform::Linux, ".mozilla/native-messaging-hosts")]
#[case::chrome_mac(
    Browser::Chrome,
    Platform::MacOs,
    "Library/Application Support/Google/Chrome/NativeMessagingHosts"
)]
#[case::edge_mac(
    Browser::Edge,
    Platform::MacOs,
    "Library/Application Support/Microsoft Edge/NativeMessagingHosts"
)]
#[case::brave_mac(
    Browser::Brave,
    Platform::MacOs,
    "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"
)]
#[case::vivaldi_mac(
    Browser::Vivaldi,
    Platform::MacOs,
    "Library/Application Support/Vivaldi/NativeMessagingHosts"
)]
#[case::firefox_mac(
    Browser::Firefox,
    Platform::MacOs,
    "Library/Application Support/Mozilla/NativeMessagingHosts"
)]
fn every_browser_lives_where_its_loader_looks(
    #[case] browser: Browser,
    #[case] platform: Platform,
    #[case] expected: &str,
) {
    let home = PathBuf::from("/home/tester");
    let location = manifest_location(browser, platform, &home, "app.castellan.host");
    assert_eq!(
        location.file(),
        Path::new("/home/tester")
            .join(expected)
            .join("app.castellan.host.json"),
        "{browser:?} on {platform:?}"
    );
}

#[test]
fn windows_manifests_are_registry_values_pointing_at_one_shared_file() {
    let home = PathBuf::from("C:/Users/tester");
    for (browser, key) in [
        (Browser::Chrome, r"Software\Google\Chrome"),
        (Browser::Edge, r"Software\Microsoft\Edge"),
        (Browser::Brave, r"Software\BraveSoftware\Brave-Browser"),
        (Browser::Vivaldi, r"Software\Vivaldi"),
        (Browser::Firefox, r"Software\Mozilla"),
    ] {
        let location = manifest_location(browser, Platform::Windows, &home, "app.castellan.host");
        match location {
            castellan_manifests::ManifestLocation::Registry { key: got, file } => {
                assert_eq!(
                    got,
                    format!(r"{key}\NativeMessagingHosts\app.castellan.host")
                );
                assert_eq!(
                    file,
                    home.join("AppData/Local/Castellan/app.castellan.host.json")
                );
            }
            other => panic!("expected a registry location, got {other:?}"),
        }
    }
}

#[test]
fn families_and_faces_map_the_way_the_protocol_expects() {
    assert_eq!(Browser::Chrome.family(), Family::Chromium);
    assert_eq!(Browser::Vivaldi.family(), Family::Chromium);
    assert_eq!(Browser::Firefox.family(), Family::Gecko);
    assert_eq!(Browser::Brave.face(), FaceKind::Brave);
    assert_eq!(Browser::Firefox.face(), FaceKind::Firefox);
}

// ── Detection ────────────────────────────────────────────────────────────────

#[test]
fn detection_finds_the_installed_and_only_those() {
    let home = home_with(Platform::Linux, &[Browser::Chrome, Browser::Firefox]);
    let installer = installer(&home, Platform::Linux);
    assert_eq!(
        installer.detect_installed(),
        vec![Browser::Chrome, Browser::Firefox]
    );
}

// ── The one-pass install ─────────────────────────────────────────────────────

#[rstest]
#[case::linux(Platform::Linux)]
#[case::macos(Platform::MacOs)]
fn one_pass_writes_every_installed_browser_and_skips_the_rest(#[case] platform: Platform) {
    let home = home_with(
        platform,
        &[Browser::Chrome, Browser::Edge, Browser::Firefox],
    );
    let installer = installer(&home, platform);
    let report = installer.install_all();

    let by_name = |outcome: &castellan_manifests::BrowserOutcome| *outcome.browser();
    let written: Vec<Browser> = report
        .outcomes
        .iter()
        .filter(|outcome| {
            matches!(
                outcome,
                castellan_manifests::BrowserOutcome::Written { .. }
                    | castellan_manifests::BrowserOutcome::AlreadyCorrect { .. }
                    | castellan_manifests::BrowserOutcome::Repaired { .. }
            )
        })
        .map(by_name)
        .collect();
    assert_eq!(
        written,
        vec![Browser::Chrome, Browser::Edge, Browser::Firefox]
    );

    let skipped: Vec<Browser> = report
        .outcomes
        .iter()
        .filter(|outcome| {
            matches!(
                outcome,
                castellan_manifests::BrowserOutcome::SkippedNotInstalled { .. }
            )
        })
        .map(by_name)
        .collect();
    assert_eq!(skipped, vec![Browser::Brave, Browser::Vivaldi]);

    // The files exist, where each loader looks, with the canonical bytes.
    for browser in [Browser::Chrome, Browser::Edge, Browser::Firefox] {
        let location = manifest_location(browser, platform, &home, "app.castellan.host");
        let contents = std::fs::read_to_string(location.file()).expect("manifest written");
        assert_eq!(contents, installer.manifest_content(browser.family()));
    }
}

#[rstest]
#[case::linux(Platform::Linux)]
#[case::macos(Platform::MacOs)]
fn every_id_the_identity_pins_lands_in_every_allow_list(#[case] platform: Platform) {
    // AC-2: the store ids (pinned at submission) and the dev id, in every
    // manifest, both families.
    let home = home_with(platform, &[Browser::Chrome, Browser::Firefox]);
    let installer = installer(&home, platform);
    installer.install_all();

    let chrome = std::fs::read_to_string(
        manifest_location(Browser::Chrome, platform, &home, "app.castellan.host").file(),
    )
    .expect("chrome manifest");
    let parsed: serde_json::Value = serde_json::from_str(&chrome).expect("valid json");
    let origins = parsed["allowed_origins"].as_array().expect("origins");
    assert!(origins.contains(&serde_json::json!("chrome-extension://EXTENSION_ID_DEV/")));
    assert!(origins.contains(&serde_json::json!("chrome-extension://EXTENSION_ID_STORE/")));

    let firefox = std::fs::read_to_string(
        manifest_location(Browser::Firefox, platform, &home, "app.castellan.host").file(),
    )
    .expect("firefox manifest");
    let parsed: serde_json::Value = serde_json::from_str(&firefox).expect("valid json");
    let extensions = parsed["allowed_extensions"].as_array().expect("extensions");
    assert!(extensions.contains(&serde_json::json!("fob-dev@castellan.app")));
    assert!(extensions.contains(&serde_json::json!("FIREFOX_EXTENSION_ID_STORE")));
}

#[test]
fn the_written_manifests_are_the_committed_templates_with_the_path_filled() {
    // The installer and the xtask templates share one shape: this pins
    // them together, so neither can drift without the test naming the
    // side that moved.
    let home = home_with(Platform::Linux, &[Browser::Chrome, Browser::Firefox]);
    let installer = installer(&home, Platform::Linux);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (family, template) in [
        (Family::Chromium, "chromium/app.castellan.host.json"),
        (Family::Gecko, "firefox/app.castellan.host.json"),
    ] {
        let template_path = root
            .join("apps/extension/native-hosts/generated")
            .join(template);
        let expected = std::fs::read_to_string(&template_path)
            .expect("committed template")
            .replace("{path}", "/opt/castellan/castellan");
        assert_eq!(installer.manifest_content(family), expected);
    }
}

// ── Repair ───────────────────────────────────────────────────────────────────

#[rstest]
#[case::linux(Platform::Linux)]
#[case::macos(Platform::MacOs)]
fn repair_is_idempotent_and_reports_what_it_fixed(#[case] platform: Platform) {
    let home = home_with(platform, &[Browser::Chrome]);
    let installer = installer(&home, platform);
    let identity = HostIdentity::generated();
    let location = manifest_location(Browser::Chrome, platform, &home, "app.castellan.host");

    // A manifest from an older app location, missing a pinned id.
    std::fs::create_dir_all(location.file().parent().expect("dir")).expect("dir");
    let stale = manifest_with(
        &identity,
        Family::Chromium,
        "/old/place/castellan",
        &["chrome-extension://EXTENSION_ID_DEV/".to_string()],
    );
    std::fs::write(location.file(), stale).expect("stale manifest");

    let report = installer.install_all();
    match &report.outcomes[0] {
        castellan_manifests::BrowserOutcome::Repaired { changes, .. } => {
            assert!(changes.contains(&castellan_manifests::Change::PathUpdated {
                from: "/old/place/castellan".to_string(),
                to: "/opt/castellan/castellan".to_string(),
            }));
            assert!(
                changes.contains(&castellan_manifests::Change::AllowListUpdated {
                    missing: vec!["chrome-extension://EXTENSION_ID_STORE/".to_string()],
                })
            );
            assert_eq!(report.fixed(), vec![&Browser::Chrome]);
        }
        other => panic!("expected a repair, got {other:?}"),
    }

    // The second pass has nothing to do: idempotent by construction.
    let again = installer.install_all();
    assert!(matches!(
        again.outcomes[0],
        castellan_manifests::BrowserOutcome::AlreadyCorrect { .. }
    ));
    assert!(again.fixed().is_empty());
}

#[test]
fn repair_replaces_garbage_and_foreign_manifests_and_says_so() {
    let home = home_with(Platform::Linux, &[Browser::Chrome, Browser::Brave]);
    let installer = installer(&home, Platform::Linux);
    let chrome = manifest_location(
        Browser::Chrome,
        Platform::Linux,
        &home,
        "app.castellan.host",
    );
    let brave = manifest_location(Browser::Brave, Platform::Linux, &home, "app.castellan.host");
    std::fs::create_dir_all(chrome.file().parent().expect("dir")).expect("dir");
    std::fs::write(chrome.file(), "{not json").expect("garbage");
    std::fs::create_dir_all(brave.file().parent().expect("dir")).expect("dir");
    std::fs::write(
        brave.file(),
        r#"{"name":"com.someone.else","description":"not us","path":"/x","type":"stdio","allowed_origins":[]}"#,
    )
    .expect("foreign manifest");

    let report = installer.install_all();
    match &report.outcomes[0] {
        castellan_manifests::BrowserOutcome::Repaired { changes, .. } => {
            assert_eq!(
                changes,
                &vec![castellan_manifests::Change::Replaced {
                    why: format!("{} was not valid JSON", chrome.file().display()),
                }]
            );
        }
        other => panic!("expected a garbage repair, got {other:?}"),
    }
    match &report.outcomes[2] {
        castellan_manifests::BrowserOutcome::Repaired { changes, .. } => {
            assert_eq!(
                changes,
                &vec![castellan_manifests::Change::Replaced {
                    why: "the file carried com.someone.else manifest".to_string(),
                }]
            );
        }
        other => panic!("expected a foreign repair, got {other:?}"),
    }
}

proptest::proptest! {
    #[test]
    fn repair_reaches_the_fixpoint_from_any_file_contents(
        junk in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..256)
    ) {
        // Whatever bytes are sitting at the manifest path — the property
        // says one install pass lands on canonical, and the next is a
        // no-op. That is the whole promise of the repair button.
        let home = home_with(Platform::Linux, &[Browser::Chrome]);
        let installer = installer(&home, Platform::Linux);
        let location = manifest_location(Browser::Chrome, Platform::Linux, &home, "app.castellan.host");
        std::fs::create_dir_all(location.file().parent().expect("dir")).expect("dir");
        std::fs::write(location.file(), &junk).expect("junk manifest");

        let first = installer.install_all();
        assert!(matches!(
            first.outcomes[0],
            castellan_manifests::BrowserOutcome::Written { .. }
                | castellan_manifests::BrowserOutcome::Repaired { .. }
        ));
        assert_eq!(
            std::fs::read_to_string(location.file()).expect("repaired"),
            installer.manifest_content(Family::Chromium)
        );

        let second = installer.install_all();
        assert!(matches!(
            second.outcomes[0],
            castellan_manifests::BrowserOutcome::AlreadyCorrect { .. }
        ));
    }
}

// ── The audit ────────────────────────────────────────────────────────────────

#[test]
fn a_fresh_install_audits_clean() {
    let home = home_with(Platform::Linux, &[Browser::Chrome, Browser::Firefox]);
    let installer = installer(&home, Platform::Linux);
    installer.install_all();
    assert!(installer.audit().is_empty());
}

#[test]
fn the_audit_names_every_way_a_manifest_can_be_stale() {
    use ManifestProblemKind::{Foreign, Missing, MissingId, StalePath, Unreadable};

    let home = home_with(Platform::Linux, &[Browser::Chrome]);
    let installer = installer(&home, Platform::Linux);
    let identity = HostIdentity::generated();
    let location = manifest_location(
        Browser::Chrome,
        Platform::Linux,
        &home,
        "app.castellan.host",
    );
    let path = location.file().to_path_buf();
    std::fs::create_dir_all(path.parent().expect("dir")).expect("dir");

    // Missing.
    assert_eq!(installer.audit()[0].kind, Missing);

    // Unreadable.
    std::fs::write(&path, "{oops").expect("junk");
    assert_eq!(installer.audit()[0].kind, Unreadable);

    // Foreign.
    std::fs::write(
        &path,
        r#"{"name":"com.other","description":"x","path":"/x","type":"stdio","allowed_origins":[]}"#,
    )
    .expect("foreign");
    assert_eq!(installer.audit()[0].kind, Foreign);

    // Stale path (the app moved) and a missing id, both at once.
    std::fs::write(
        &path,
        manifest_with(
            &identity,
            Family::Chromium,
            "/old/place/castellan",
            &["chrome-extension://EXTENSION_ID_DEV/".to_string()],
        ),
    )
    .expect("stale");
    let problems = installer.audit();
    assert_eq!(problems.len(), 2);
    assert_eq!(problems[0].kind, StalePath);
    assert_eq!(problems[1].kind, MissingId);
    assert!(problems[0].detail.contains("/old/place/castellan"));
    assert!(problems[1].detail.contains("EXTENSION_ID_STORE"));

    // And the repair clears the audit.
    installer.install_all();
    assert!(installer.audit().is_empty());
}

// ── Windows, honestly ────────────────────────────────────────────────────────

#[test]
fn windows_reports_unsupported_instead_of_half_installing() {
    let home = home_with(Platform::Windows, &[Browser::Chrome]);
    let installer = installer(&home, Platform::Windows);
    let report = installer.install_all();
    match &report.outcomes[0] {
        castellan_manifests::BrowserOutcome::Unsupported { .. } => {}
        other => panic!("expected Unsupported, got {other:?}"),
    }
    // Nothing was written: a manifest file without its registry value is
    // integration that only looks installed.
    let location = manifest_location(
        Browser::Chrome,
        Platform::Windows,
        &home,
        "app.castellan.host",
    );
    assert!(!location.file().exists());
    // The audit does not claim what it could not check.
    assert!(installer.audit().is_empty());
}
