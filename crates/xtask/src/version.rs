//! The workspace version, from the one place it is written down.
//!
//! `[workspace.package] version` in the root Cargo.toml is the single source
//! of truth: every crate inherits it, and everything else derives it rather
//! than restating it. convco (`.versionrc`) bumps that table; this command
//! is how scripts and workflows read it back without parsing TOML themselves.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Read `[workspace.package].version` from `<root>/Cargo.toml`, parsed as
/// TOML rather than matched by line shape, so a dependency pin or a member
/// manifest can never be mistaken for the declared version.
pub(crate) fn workspace_version(root: &Path) -> Result<String> {
    let manifest_path = root.join("Cargo.toml");
    let manifest: toml::Table = fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?
        .parse()
        .with_context(|| format!("parsing {}", manifest_path.display()))?;
    let Some(version) = manifest
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    else {
        bail!(
            "no [workspace.package] version in {}",
            manifest_path.display()
        );
    };
    Ok(version.to_string())
}

/// `xtask version`: print the bare semver (no leading `v`, no trailing
/// chatter), so `$(cargo run -q -p castellan-xtask -- version)` is a plain
/// command substitution.
pub(crate) fn print_version(root: &Path) -> Result<()> {
    print!("{}", workspace_version(root)?);
    Ok(())
}

// A bin crate exports nothing an integration test could import, so its unit
// tests live here (the one place the tests/-directory convention explicitly
// allows). They run against tempdir fixtures, never against this checkout.
#[cfg(test)]
mod tests {
    use super::workspace_version;
    use std::fs;

    /// A tempdir without pulling a dependency in for one call: unique per
    /// process, created under the OS temp dir, cleaned by the caller.
    fn fixture_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "castellan-xtask-tests-{}-{name}",
            std::process::id(),
        ));
        fs::create_dir_all(&dir).expect("fixture dir");
        dir
    }

    #[test]
    fn the_workspace_package_version_is_read_not_the_first_version_line() {
        let dir = fixture_dir("workspace-version");
        fs::write(
            dir.join("Cargo.toml"),
            "[workspace.dependencies]\nserde = { version = \"1\" }\n\n[workspace.package]\nversion = \"9.8.7\"\n",
        )
        .expect("manifest");
        assert_eq!(workspace_version(&dir).expect("version"), "9.8.7");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_manifest_without_the_table_is_an_error_naming_the_file() {
        let dir = fixture_dir("missing-table");
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").expect("manifest");
        let err = workspace_version(&dir).expect_err("must fail");
        assert!(format!("{err:#}").contains("[workspace.package]"));
        let _ = fs::remove_dir_all(&dir);
    }
}
