//! `otpauth://` parsing and RFC 6238 generation, through the public API:
//! the same functions every face calls, from the same crate.

use castellan_otp::{OtpError, TotpConfig, code_at, code_now, parse};
use data_encoding::BASE32_NOPAD;
use proptest::prelude::*;
use rstest::{fixture, rstest};

/// RFC 6238's SHA-1 test secret: base32 of "12345678901234567890".
const RFC_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

/// The RFC's reference seed, parsed once and injected into every test
/// that wants it — the pytest-fixture pattern, via rstest.
#[fixture]
fn rfc_config() -> TotpConfig {
    parse(&format!("otpauth://totp/Test:alice?secret={RFC_SECRET}")).unwrap()
}

#[rstest]
fn parses_path_issuer_and_account(rfc_config: TotpConfig) {
    assert_eq!(rfc_config.issuer.as_deref(), Some("Test"));
    assert_eq!(rfc_config.account, "alice");
    assert_eq!(rfc_config.digits, 6);
    assert_eq!(rfc_config.period, 30);
}

#[rstest]
#[case(6, "287082")]
#[case(7, "4287082")]
#[case(8, "94287082")]
/// RFC 6238 Appendix B at T=59 (counter 1): every digit count is a
/// truncation of the same dynamic-truncation value, 1094287082.
fn rfc6238_vectors_by_digit_count(
    rfc_config: TotpConfig,
    #[case] digits: u32,
    #[case] expected: &str,
) {
    let mut config = rfc_config;
    config.digits = digits;
    assert_eq!(code_at(&config, 1), expected);
}

#[rstest]
fn code_now_reports_remaining_seconds(rfc_config: TotpConfig) {
    let (code, remaining) = code_now(&rfc_config, 59);
    assert_eq!(code, "287082");
    assert_eq!(remaining, 1);
}

#[rstest]
#[case("Path:alice", Some("Param"), Some("Param"))]
#[case("Path:alice", None, Some("Path"))]
#[case("alice", Some("Param"), Some("Param"))]
#[case("alice", None, None)]
/// The `issuer` parameter wins over the path issuer: providers that set
/// both have the parameter as the source of truth in practice.
fn issuer_precedence(
    #[case] label: &str,
    #[case] issuer_param: Option<&str>,
    #[case] expected: Option<&str>,
) {
    let uri = match issuer_param {
        Some(issuer) => {
            format!("otpauth://totp/{label}?secret={RFC_SECRET}&issuer={issuer}")
        }
        None => format!("otpauth://totp/{label}?secret={RFC_SECRET}"),
    };
    assert_eq!(parse(&uri).unwrap().issuer.as_deref(), expected);
}

#[rstest]
#[case("algorithm=SHA256", "algorithm")]
#[case("algorithm=sha512", "algorithm")]
#[case("encoder=steam", "encoder")]
/// Refused with the parameter named, never silently ignored: a skipped
/// parameter is a code the user reads, types, and watches get rejected.
fn unsupported_parameters_are_refused(#[case] param: &str, #[case] name: &str) {
    let err = parse(&format!(
        "otpauth://totp/Test:alice?secret={RFC_SECRET}&{param}"
    ))
    .unwrap_err();
    assert!(matches!(err, OtpError::UnsupportedParameter { name: n, .. } if n == name));
}

#[rstest]
#[case("SHA1")]
#[case("sha1")]
fn sha1_algorithm_is_accepted(#[case] value: &str) {
    assert!(
        parse(&format!(
            "otpauth://totp/Test:alice?secret={RFC_SECRET}&algorithm={value}"
        ))
        .is_ok()
    );
}

#[rstest]
#[case("digits=abc")]
#[case("period=-1")]
fn non_numeric_parameters_are_rejected(#[case] param: &str) {
    let err = parse(&format!(
        "otpauth://totp/Test:alice?secret={RFC_SECRET}&{param}"
    ))
    .unwrap_err();
    assert!(matches!(err, OtpError::BadNumber { .. }));
}

#[test]
fn rejects_hotp_and_garbage() {
    assert!(matches!(
        parse("otpauth://hotp/Test:alice?secret=AAA"),
        Err(OtpError::NotOtpauth(_))
    ));
    assert!(matches!(
        parse("otpauth://totp/Test:alice"),
        Err(OtpError::MissingSecret)
    ));
}

/// Percent-encode everything outside the URI unreserved set — the
/// inverse of the crate's `percent_decode`, for building round-trip
/// fixtures.
fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Issuer text drawn from a charset that includes the characters that
/// need escaping in a query parameter (space, colon).
fn issuer_text() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<u8>(), 0..16).prop_map(|bytes| {
        const CHARSET: &[u8] = b"abcxyz :.-019";
        bytes
            .iter()
            .map(|b| CHARSET[usize::from(*b) % CHARSET.len()] as char)
            .collect()
    })
}

/// Account text for the path label: no colon (it would split
/// issuer from account) and no percent sign (the path is not decoded).
fn account_text() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<u8>(), 1..16).prop_map(|bytes| {
        const CHARSET: &[u8] = b"abcxyz._-019";
        bytes
            .iter()
            .map(|b| CHARSET[usize::from(*b) % CHARSET.len()] as char)
            .collect()
    })
}

proptest! {
    /// Whatever a provider's export URI carries, `parse` recovers it:
    /// the fields the user sees (issuer, account) and the ones the code
    /// depends on (secret, digits, period).
    #[test]
    fn otpauth_uris_round_trip(
        secret in prop::collection::vec(any::<u8>(), 1..64),
        issuer in issuer_text(),
        account in account_text(),
        digits in 6u32..=8,
        period in 15u32..=120,
    ) {
        let uri = format!(
            "otpauth://totp/{account}?secret={}&issuer={}&digits={digits}&period={period}",
            BASE32_NOPAD.encode(&secret),
            percent_encode(&issuer),
        );
        let config = parse(&uri).unwrap();
        prop_assert_eq!(config.secret, secret);
        prop_assert_eq!(config.issuer.as_deref(), Some(issuer.as_str()));
        prop_assert_eq!(config.account, account);
        prop_assert_eq!(config.digits, digits);
        prop_assert_eq!(config.period, period);
    }

    /// The code is exactly `digits` long, for any secret and counter —
    /// a leading zero must survive, not be parsed away.
    #[test]
    fn code_length_is_the_requested_digits(
        secret in prop::collection::vec(any::<u8>(), 1..64),
        digits in 1u32..=9,
        counter in any::<u64>(),
    ) {
        let config = TotpConfig {
            secret,
            digits,
            period: 30,
            issuer: None,
            account: "alice".into(),
        };
        prop_assert_eq!(code_at(&config, counter).len(), digits as usize);
    }

    /// `code_now` agrees with `code_at` on the current step and reports
    /// the exact distance to the next one — the number the UI countdown
    /// in every face runs on. `now` is bounded to 2^40 seconds (~year
    /// 34,000): the step arithmetic assumes a clock a machine could
    /// actually have, not the u64 heat death.
    #[test]
    fn code_now_tracks_the_step_boundaries(
        secret in prop::collection::vec(any::<u8>(), 1..64),
        period in 15u32..=60,
        now in 0u64..(1u64 << 40),
    ) {
        let config = TotpConfig {
            secret,
            digits: 6,
            period,
            issuer: None,
            account: "alice".into(),
        };
        let (code, remaining) = code_now(&config, now);
        prop_assert_eq!(code, code_at(&config, now / u64::from(period)));
        prop_assert_eq!(
            remaining,
            period - u32::try_from(now % u64::from(period)).unwrap(),
        );
    }
}
