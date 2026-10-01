//! `otpauth://` URIs and RFC 6238 TOTP codes.
//!
//! This is the first piece of "same scoped logic in every face": the desktop
//! and mobile apps compute codes through this crate, and the extension runs
//! the same parsing and generation in-browser through `castellan-wasm`. The
//! authenticator is therefore never re-implemented in TypeScript.
//!
//! Scope is deliberately narrow. SHA-1 only, because that is what the
//! overwhelming majority of real-world `otpauth` seeds specify; `algorithm`
//! parameters other than SHA1 are refused with a clear error rather than
//! silently computing the wrong code. Steam and HOTP are future work with a
//! home waiting for them here.

use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, KeyInit, Mac};
use thiserror::Error;

/// A parsed `otpauth://` seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpConfig {
    /// The decoded shared secret.
    pub secret: Vec<u8>,
    /// Code length in digits (6 and 8 are what services actually use).
    pub digits: u32,
    /// Step in seconds; 30 unless the URI said otherwise.
    pub period: u32,
    /// Issuer from the URI path (`Issuer:account`) or `issuer` parameter.
    pub issuer: Option<String>,
    /// The account name, i.e. what the code belongs to.
    pub account: String,
}

/// Everything that can go wrong while reading a seed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OtpError {
    /// Not an `otpauth://totp/...` URI.
    #[error("not an otpauth://totp URI: {0}")]
    NotOtpauth(String),
    /// The URI had no `secret` parameter — nothing can be derived from it.
    #[error("missing secret parameter")]
    MissingSecret,
    /// The `secret` parameter was not valid base32.
    #[error("secret is not valid base32")]
    BadSecret,
    /// A parameter had a value the parser refused.
    #[error("unsupported parameter {name}: {value}")]
    UnsupportedParameter {
        /// Which parameter.
        name: String,
        /// The refused value.
        value: String,
    },
    /// A numeric parameter was not a number.
    #[error("parameter {name} is not a number: {value}")]
    BadNumber {
        /// Which parameter.
        name: String,
        /// The value that was not a number.
        value: String,
    },
}

/// Parse an `otpauth://totp/...` URI.
///
/// Done by hand rather than with a URL crate: `otpauth` is not a registered
/// scheme, parsers disagree about what to do with it, and the grammar is
/// small enough that a reader of this function knows the whole format.
pub fn parse(uri: &str) -> Result<TotpConfig, OtpError> {
    let rest = uri
        .strip_prefix("otpauth://totp/")
        .ok_or_else(|| OtpError::NotOtpauth(uri.to_string()))?;

    let (path, query) = match rest.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (rest, None),
    };

    // The label is `Issuer:account` or just `account`; both may be
    // percent-encoded, and only the issuer half may contain the colon.
    let (issuer_from_path, account) = match path.split_once(':') {
        Some((issuer, account)) => (Some(issuer.to_string()), account.to_string()),
        None => (None, path.to_string()),
    };

    let mut secret = None;
    let mut digits = 6u32;
    let mut period = 30u32;
    let mut issuer_param = None;

    if let Some(query) = query {
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            let value = percent_decode(value);
            match key {
                "secret" => secret = Some(value),
                "digits" => {
                    digits = value.parse().map_err(|_| OtpError::BadNumber {
                        name: "digits".into(),
                        value,
                    })?;
                }
                "period" => {
                    period = value.parse().map_err(|_| OtpError::BadNumber {
                        name: "period".into(),
                        value,
                    })?;
                }
                "issuer" => issuer_param = Some(value),
                // Refused, not ignored: an algorithm we skip is a code the
                // user reads and types, then watches get rejected.
                "algorithm" if !value.eq_ignore_ascii_case("SHA1") => {
                    return Err(OtpError::UnsupportedParameter {
                        name: "algorithm".into(),
                        value,
                    });
                }
                "encoder" => {
                    return Err(OtpError::UnsupportedParameter {
                        name: "encoder".into(),
                        value,
                    });
                }
                _ => {}
            }
        }
    }

    let secret = secret.ok_or(OtpError::MissingSecret)?;
    let secret = BASE32_NOPAD
        .decode(secret.to_ascii_uppercase().replace('=', "").as_bytes())
        .map_err(|_| OtpError::BadSecret)?;
    if secret.is_empty() {
        return Err(OtpError::BadSecret);
    }

    Ok(TotpConfig {
        secret,
        digits,
        period,
        // The parameter wins over the path: providers that set both have the
        // parameter as the source of truth in practice.
        issuer: issuer_param.or(issuer_from_path),
        account,
    })
}

/// The TOTP code for a counter value (Unix time ÷ period).
///
/// Exposed separately from [`code_now`] so tests can pin the RFC 6238 test
/// vectors instead of freezing the clock.
pub fn code_at(config: &TotpConfig, counter: u64) -> String {
    let mut mac = Hmac::<sha1::Sha1>::new_from_slice(&config.secret)
        .expect("HMAC accepts keys of any length");
    mac.update(&counter.to_be_bytes());
    let digest = mac.finalize().into_bytes();

    // RFC 4226 dynamic truncation.
    let offset = (digest[digest.len() - 1] & 0x0f) as usize;
    let binary = u32::from_be_bytes([
        digest[offset] & 0x7f,
        digest[offset + 1],
        digest[offset + 2],
        digest[offset + 3],
    ]);
    let modulus = 10u32.checked_pow(config.digits).unwrap_or(u32::MAX).max(1);
    let code = binary % modulus;
    format!("{code:0width$}", width = config.digits as usize)
}

/// The code that is valid at `now_unix`, and how long it stays valid.
pub fn code_now(config: &TotpConfig, now_unix: u64) -> (String, u32) {
    let counter = now_unix / u64::from(config.period);
    let code = code_at(config, counter);
    let next_step = (counter + 1) * u64::from(config.period);
    let seconds_remaining = u32::try_from(next_step - now_unix).unwrap_or(config.period);
    (code, seconds_remaining)
}

/// Decode the few percent-escapes that actually appear in otpauth URIs.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() + 1 && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
