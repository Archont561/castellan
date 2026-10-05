//! One front door for every "bring your codes over" payload: pasted
//! `otpauth://` URIs, Google Authenticator migration QRs, and the plain
//! (unencrypted) Aegis and andOTP JSON exports.
//!
//! The sniffing lives here, not in the faces: the desktop paste box, a
//! decoded QR frame and a dropped export file all land in [`preview`],
//! and the extension will reach the same function through the WASM build
//! when task-19 arrives. Encrypted exports are *named* and refused — "I
//! can see what this is, task-24 adds the decryption" is a better answer
//! than a parse error.

use serde::Deserialize;

use crate::{AccountCandidate, OtpError, migration, otpauth_uri};

/// Sniff `payload` and return per-account candidates for review.
///
/// Accepts, in sniffing order: `otpauth-migration://` URIs,
/// `otpauth://` URIs (one per line — several QRs pasted together stay
/// one batch), Aegis plain JSON exports, and andOTP plain JSON exports.
/// Anything else is [`OtpError::UnrecognizedImport`].
pub fn preview(payload: &str) -> Result<Vec<AccountCandidate>, OtpError> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Err(OtpError::UnrecognizedImport);
    }
    if trimmed.starts_with("otpauth-migration://") {
        return migration::parse_migration(trimmed);
    }
    if trimmed.starts_with("otpauth://") {
        return Ok(trimmed
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(candidate_from_uri)
            .collect());
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(result) = aegis(&value) {
            return result;
        }
        if let Some(result) = andotp(&value) {
            return result;
        }
    }
    Err(OtpError::UnrecognizedImport)
}

/// Validate one pasted/decoded URI into a candidate. The URI is kept
/// verbatim when it parses — the user scanned it, so what they scanned
/// is what gets stored (`NewEntry.otpauth` keeps the same rule).
fn candidate_from_uri(uri: &str) -> AccountCandidate {
    match crate::parse(uri) {
        Ok(config) => AccountCandidate {
            issuer: config.issuer,
            account: config.account,
            otpauth: Some(uri.to_string()),
            problem: None,
        },
        Err(error) => AccountCandidate {
            issuer: None,
            account: label_of(uri),
            otpauth: None,
            problem: Some(error.to_string()),
        },
    }
}

/// The label half of an unparseable URI, so a problem row still names
/// which account it is about.
fn label_of(uri: &str) -> String {
    uri.split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(uri)
        .split_once('/')
        .map(|(_, rest)| rest)
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .to_string()
}

/// One entry of an Aegis plain export.
#[derive(Debug, Deserialize)]
struct AegisEntry {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    issuer: Option<String>,
    info: AegisInfo,
}

#[derive(Debug, Deserialize)]
struct AegisInfo {
    secret: String,
    #[serde(default = "default_algo")]
    algo: String,
    #[serde(default = "default_digits")]
    digits: u32,
    #[serde(default = "default_period")]
    period: u32,
}

fn default_algo() -> String {
    "SHA1".into()
}
const fn default_digits() -> u32 {
    6
}
const fn default_period() -> u32 {
    30
}

/// Aegis: `{"header": {...}, "db": {...}}`. An encrypted export carries
/// `db` as a base64 *string* instead of an object — recognized and
/// refused by name (task-24 adds the AES-GCM slot decryption).
fn aegis(value: &serde_json::Value) -> Option<Result<Vec<AccountCandidate>, OtpError>> {
    let db = value.get("db")?;
    if db.is_string() {
        return Some(Err(OtpError::EncryptedExport("Aegis".into())));
    }
    let entries = db.get("entries")?.as_array()?;
    let accounts = entries
        .iter()
        .map(|entry| match AegisEntry::deserialize(entry) {
            Ok(entry) => {
                let issuer = entry.issuer.filter(|issuer| !issuer.is_empty());
                candidate_from_parts(
                    issuer,
                    entry.name,
                    &entry.kind,
                    &entry.info.secret,
                    &entry.info.algo,
                    entry.info.digits,
                    entry.info.period,
                )
            }
            Err(error) => AccountCandidate {
                issuer: None,
                account: "(unreadable entry)".into(),
                otpauth: None,
                problem: Some(format!("entry does not match the Aegis layout: {error}")),
            },
        })
        .collect();
    Some(Ok(accounts))
}

/// One entry of an andOTP plain export (the whole file is a JSON array).
#[derive(Debug, Deserialize)]
struct AndOtpEntry {
    secret: String,
    label: String,
    #[serde(default)]
    issuer: Option<String>,
    #[serde(rename = "type", default = "default_type")]
    kind: String,
    #[serde(default = "default_algo")]
    algorithm: String,
    #[serde(default = "default_digits")]
    digits: u32,
    #[serde(default = "default_period")]
    period: u32,
}

fn default_type() -> String {
    "TOTP".into()
}

/// andOTP plain export: a bare array of entries. The encrypted variant
/// is a binary blob, never valid JSON, so it cannot reach here — it
/// fails the JSON parse in [`preview`] and reports as unrecognized.
fn andotp(value: &serde_json::Value) -> Option<Result<Vec<AccountCandidate>, OtpError>> {
    let entries = value.as_array()?;
    // An empty array is JSON but proves nothing about the format; and a
    // first element without andOTP's mandatory fields means this is some
    // other tool's array.
    if entries.is_empty() || entries[0].get("secret").is_none() || entries[0].get("label").is_none()
    {
        return None;
    }
    let accounts = entries
        .iter()
        .map(|entry| match AndOtpEntry::deserialize(entry) {
            Ok(entry) => {
                let issuer = entry.issuer.filter(|issuer| !issuer.is_empty());
                candidate_from_parts(
                    issuer,
                    entry.label,
                    &entry.kind,
                    &entry.secret,
                    &entry.algorithm,
                    entry.digits,
                    entry.period,
                )
            }
            Err(error) => AccountCandidate {
                issuer: None,
                account: "(unreadable entry)".into(),
                otpauth: None,
                problem: Some(format!("entry does not match the andOTP layout: {error}")),
            },
        })
        .collect();
    Some(Ok(accounts))
}

/// The shared policy gate for file-based importers: TOTP + SHA-1 imports,
/// everything else is named with the reason it cannot (yet).
fn candidate_from_parts(
    issuer: Option<String>,
    account: String,
    kind: &str,
    secret: &str,
    algorithm: &str,
    digits: u32,
    period: u32,
) -> AccountCandidate {
    let problem = if !kind.eq_ignore_ascii_case("totp") {
        Some(format!(
            "{} accounts need task-42's encoder work",
            kind.to_uppercase()
        ))
    } else if !algorithm.eq_ignore_ascii_case("sha1") {
        Some("only SHA-1 codes can be computed by this build (task-42)".to_string())
    } else {
        None
    };
    if let Some(problem) = problem {
        return AccountCandidate {
            issuer,
            account,
            otpauth: None,
            problem: Some(problem),
        };
    }

    let uri = otpauth_uri(
        issuer.as_deref(),
        &account,
        &secret.replace(' ', "").to_ascii_uppercase(),
        digits,
        period,
    );
    // Round-trip through the real parser: a bad secret or digit count is
    // caught by the same code that will compute the codes later.
    match crate::parse(&uri) {
        Ok(_) => AccountCandidate {
            issuer,
            account,
            otpauth: Some(uri),
            problem: None,
        },
        Err(error) => AccountCandidate {
            issuer,
            account,
            otpauth: None,
            problem: Some(error.to_string()),
        },
    }
}
