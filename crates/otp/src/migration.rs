//! `otpauth-migration://offline?data=...` — the Google Authenticator
//! export format: a base64 protobuf carrying a batch of accounts.
//!
//! The protobuf reader here is hand-rolled, like the `otpauth` parser in
//! `lib.rs`, and for a sharper reason: the airlocked toolchain vendors no
//! protobuf crate, and the message is two levels deep with seven scalar
//! fields — a schema small enough that the whole wire format fits in one
//! screen of code. The field numbers are pinned by Google's exporter and
//! documented in every independent reimplementation (Aegis, 2FAS,
//! `extract_otp_secrets`); they cannot change without breaking Google's
//! own import path.

use data_encoding::BASE32_NOPAD;

use crate::{AccountCandidate, OtpError, otpauth_uri, percent_decode};

/// One account as the migration payload carries it, before policy is
/// applied. Field numbers from Google's `OtpParameters` message.
#[derive(Debug, Default, Clone)]
struct RawParameters {
    /// Field 1: the shared secret, raw bytes (not base32).
    secret: Vec<u8>,
    /// Field 2: the label, usually `Issuer:account`.
    name: String,
    /// Field 3: the issuer, when the exporter set one.
    issuer: Option<String>,
    /// Field 4: 0 unspecified, 1 SHA1, 2 SHA256, 3 SHA512, 4 MD5.
    algorithm: u64,
    /// Field 5: 0 unspecified, 1 six digits, 2 eight digits.
    digits: u64,
    /// Field 6: 0 unspecified, 1 HOTP, 2 TOTP.
    otp_type: u64,
}

/// Parse a Google Authenticator export URI into per-account candidates.
///
/// Accounts this build can compute (TOTP, SHA-1, 6 or 8 digits) come back
/// with a canonical `otpauth://` URI; the rest come back named, with a
/// `problem` explaining why they are review-only — a batch import must
/// show the user every account the QR carried, not silently shrink it.
pub fn parse_migration(uri: &str) -> Result<Vec<AccountCandidate>, OtpError> {
    let rest = uri
        .strip_prefix("otpauth-migration://offline?")
        .ok_or_else(|| OtpError::NotOtpauth(uri.to_string()))?;

    let mut data = None;
    for pair in rest.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "data" {
            data = Some(percent_decode(value));
        }
    }
    let data = data.ok_or_else(|| OtpError::BadMigration("missing data parameter".into()))?;

    // QR scanners hand over raw standard base64; anything that went
    // through URL machinery may carry the URL-safe alphabet or spaces
    // where `+` stood. Normalize instead of guessing the variant.
    let normalized: String = data
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            ' ' => '+',
            '-' => '+',
            '_' => '/',
            other => other,
        })
        .collect();
    let normalized = normalized.trim_end_matches('=');
    let bytes = data_encoding::BASE64_NOPAD
        .decode(normalized.as_bytes())
        .map_err(|_| OtpError::BadMigration("data is not valid base64".into()))?;

    let mut accounts = Vec::new();
    let mut reader = Reader::new(&bytes);
    while let Some((field, wire)) = reader.tag()? {
        if field == 1 && wire == 2 {
            let message = reader.length_delimited()?;
            accounts.push(candidate(parse_parameters(message)?));
        } else {
            reader.skip(wire)?;
        }
    }
    if accounts.is_empty() {
        return Err(OtpError::BadMigration("no accounts in payload".into()));
    }
    Ok(accounts)
}

/// Apply this build's policy to one raw account: TOTP + SHA-1 + 6/8
/// digits imports; everything else is named and refused with a reason.
fn candidate(raw: RawParameters) -> AccountCandidate {
    // The label is `Issuer:account` when the issuer field is absent —
    // same rule as the otpauth parser, parameter wins over path.
    let (issuer_from_name, account) = match raw.name.split_once(':') {
        Some((issuer, account)) => (Some(issuer.to_string()), account.to_string()),
        None => (None, raw.name.clone()),
    };
    let issuer = raw.issuer.clone().or(issuer_from_name);

    let problem = if raw.secret.is_empty() {
        Some("account carries no secret".to_string())
    } else if raw.otp_type == 1 {
        Some("HOTP accounts need counter support (task-42)".to_string())
    } else if raw.algorithm > 1 {
        Some("only SHA-1 codes can be computed by this build (task-42)".to_string())
    } else if raw.digits > 2 {
        Some("unsupported digit count".to_string())
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

    let digits = if raw.digits == 2 { 8 } else { 6 };
    let secret_b32 = BASE32_NOPAD.encode(&raw.secret);
    let uri = otpauth_uri(issuer.as_deref(), &account, &secret_b32, digits, 30);
    AccountCandidate {
        issuer,
        account,
        otpauth: Some(uri),
        problem: None,
    }
}

/// Parse one `OtpParameters` message.
fn parse_parameters(bytes: &[u8]) -> Result<RawParameters, OtpError> {
    let mut raw = RawParameters::default();
    let mut reader = Reader::new(bytes);
    while let Some((field, wire)) = reader.tag()? {
        match (field, wire) {
            (1, 2) => raw.secret = reader.length_delimited()?.to_vec(),
            (2, 2) => raw.name = String::from_utf8_lossy(reader.length_delimited()?).into_owned(),
            (3, 2) => {
                let issuer = String::from_utf8_lossy(reader.length_delimited()?).into_owned();
                if !issuer.is_empty() {
                    raw.issuer = Some(issuer);
                }
            }
            (4, 0) => raw.algorithm = reader.varint()?,
            (5, 0) => raw.digits = reader.varint()?,
            (6, 0) => raw.otp_type = reader.varint()?,
            _ => reader.skip(wire)?,
        }
    }
    Ok(raw)
}

/// A protobuf wire reader: varints and length-delimited fields, which is
/// all the migration payload uses, plus skipping for anything unknown so
/// a future exporter field cannot break the import.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// The next field tag, or `None` at a clean end of input.
    fn tag(&mut self) -> Result<Option<(u64, u8)>, OtpError> {
        if self.pos >= self.buf.len() {
            return Ok(None);
        }
        let tag = self.varint()?;
        #[allow(clippy::cast_possible_truncation)]
        Ok(Some((tag >> 3, (tag & 0b111) as u8)))
    }

    fn varint(&mut self) -> Result<u64, OtpError> {
        let mut value: u64 = 0;
        let mut shift = 0u32;
        loop {
            let byte = *self
                .buf
                .get(self.pos)
                .ok_or_else(|| OtpError::BadMigration("truncated varint".into()))?;
            self.pos += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(OtpError::BadMigration("varint overflows 64 bits".into()));
            }
        }
    }

    fn length_delimited(&mut self) -> Result<&'a [u8], OtpError> {
        let len = usize::try_from(self.varint()?)
            .map_err(|_| OtpError::BadMigration("length does not fit".into()))?;
        let end = self
            .pos
            .checked_add(len)
            .filter(|end| *end <= self.buf.len())
            .ok_or_else(|| OtpError::BadMigration("truncated field".into()))?;
        let bytes = &self.buf[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    fn skip(&mut self, wire: u8) -> Result<(), OtpError> {
        match wire {
            0 => {
                self.varint()?;
            }
            1 => self.advance(8)?,
            2 => {
                self.length_delimited()?;
            }
            5 => self.advance(4)?,
            other => {
                return Err(OtpError::BadMigration(format!(
                    "unsupported wire type {other}"
                )));
            }
        }
        Ok(())
    }

    fn advance(&mut self, by: usize) -> Result<(), OtpError> {
        if self.pos + by > self.buf.len() {
            return Err(OtpError::BadMigration("truncated field".into()));
        }
        self.pos += by;
        Ok(())
    }
}
