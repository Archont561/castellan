//! Google Authenticator migration payloads, through the public API.
//!
//! One fixed real-world vector (the one every independent importer uses
//! as its reference), plus payloads *constructed* by a test-local
//! protobuf encoder — the encoder lives here, not in the crate, so the
//! reader under test can never be validated against itself.

use castellan_otp::{OtpError, migration::parse_migration};
use data_encoding::BASE64;
use rstest::rstest;

/// The reference export QR: one account, secret "Hello!\xde\xad\xbe\xef"
/// (base32 `JBSWY3DPEHPK3PXP`), label `Example:alice@google.com`, issuer
/// `Example`, SHA1, six digits, TOTP — with the base64 padding
/// percent-encoded the way a scanned QR hands it over.
const REFERENCE: &str = "otpauth-migration://offline?data=CjEKCkhlbGxvId6tvu8SGEV4YW1wbGU6YWxpY2VAZ29vZ2xlLmNvbRoHRXhhbXBsZSABKAEwAg%3D%3D";

#[rstest]
fn the_reference_export_parses_to_one_importable_account() {
    let accounts = parse_migration(REFERENCE).unwrap();
    assert_eq!(accounts.len(), 1);
    let account = &accounts[0];
    assert_eq!(account.issuer.as_deref(), Some("Example"));
    assert_eq!(account.account, "alice@google.com");
    assert_eq!(account.problem, None);
    let uri = account.otpauth.as_deref().unwrap();
    let config = castellan_otp::parse(uri).unwrap();
    assert_eq!(
        config.secret, b"Hello!\xde\xad\xbe\xef",
        "the migration secret must survive base64 -> protobuf -> base32"
    );
    assert_eq!(config.digits, 6);
    assert_eq!(config.period, 30);
}

#[rstest]
fn the_reference_code_matches_the_classic_test_secret() {
    // JBSWY3DPEHPK3PXP at counter 1, verified against an independent
    // implementation (python hmac/sha1, dynamic truncation by hand);
    // computing it proves the imported URI is not merely well-formed but
    // generates the codes the service expects.
    let accounts = parse_migration(REFERENCE).unwrap();
    let uri = accounts[0].otpauth.as_deref().unwrap();
    let config = castellan_otp::parse(uri).unwrap();
    assert_eq!(castellan_otp::code_at(&config, 1), "996554");
}

/// Minimal protobuf writer for constructing test payloads.
mod encode {
    pub(crate) fn varint(mut value: u64, out: &mut Vec<u8>) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                out.push(byte);
                return;
            }
            out.push(byte | 0x80);
        }
    }

    pub(crate) fn bytes_field(field: u64, bytes: &[u8], out: &mut Vec<u8>) {
        varint(field << 3 | 2, out);
        varint(bytes.len() as u64, out);
        out.extend_from_slice(bytes);
    }

    pub(crate) fn varint_field(field: u64, value: u64, out: &mut Vec<u8>) {
        varint(field << 3, out);
        varint(value, out);
    }
}

/// One OtpParameters message with the given scalars.
fn parameters(name: &str, issuer: &str, algorithm: u64, digits: u64, otp_type: u64) -> Vec<u8> {
    let mut out = Vec::new();
    encode::bytes_field(1, b"12345678901234567890", &mut out);
    encode::bytes_field(2, name.as_bytes(), &mut out);
    if !issuer.is_empty() {
        encode::bytes_field(3, issuer.as_bytes(), &mut out);
    }
    encode::varint_field(4, algorithm, &mut out);
    encode::varint_field(5, digits, &mut out);
    encode::varint_field(6, otp_type, &mut out);
    out
}

fn payload_uri(accounts: &[Vec<u8>]) -> String {
    let mut payload = Vec::new();
    for account in accounts {
        encode::bytes_field(1, account, &mut payload);
    }
    // Trailing exporter metadata (version, batch_size, batch_index,
    // batch_id) the reader must skip without understanding.
    encode::varint_field(2, 1, &mut payload);
    encode::varint_field(3, 1, &mut payload);
    encode::varint_field(4, 0, &mut payload);
    encode::varint_field(5, 1_234_567, &mut payload);
    format!(
        "otpauth-migration://offline?data={}",
        BASE64
            .encode(&payload)
            .replace('+', "%2B")
            .replace('/', "%2F")
            .replace('=', "%3D")
    )
}

#[rstest]
fn a_batch_keeps_every_account_and_names_the_refused_ones() {
    let uri = payload_uri(&[
        parameters("GitHub:octocat", "GitHub", 1, 1, 2),
        parameters("Bank:alice", "Bank", 2, 1, 2), // SHA256: refused
        parameters("Legacy:bob", "Legacy", 1, 1, 1), // HOTP: refused
        parameters("Wide:carol", "Wide", 1, 2, 2), // eight digits: fine
    ]);
    let accounts = parse_migration(&uri).unwrap();
    assert_eq!(accounts.len(), 4, "review must see refused accounts too");

    assert_eq!(accounts[0].problem, None);
    assert!(accounts[1].problem.as_deref().unwrap().contains("SHA-1"));
    assert!(accounts[2].problem.as_deref().unwrap().contains("HOTP"));
    assert_eq!(accounts[3].problem, None);

    let wide = castellan_otp::parse(accounts[3].otpauth.as_deref().unwrap()).unwrap();
    assert_eq!(wide.digits, 8);
    assert_eq!(wide.issuer.as_deref(), Some("Wide"));
}

#[rstest]
fn url_safe_base64_is_accepted() {
    // The same payload, carried through URL machinery that swapped the
    // alphabet: scanners and share sheets both produce this in the wild.
    let uri = payload_uri(&[parameters("A:b", "A", 1, 1, 2)]);
    let data = uri.split_once("data=").unwrap().1;
    let url_safe = data
        .replace("%2B", "-")
        .replace("%2F", "_")
        .replace("%3D", "");
    let accounts =
        parse_migration(&format!("otpauth-migration://offline?data={url_safe}")).unwrap();
    assert_eq!(accounts[0].problem, None);
}

#[rstest]
#[case("otpauth-migration://offline?data=", "missing/empty data")]
#[case("otpauth-migration://offline?data=%%%", "not base64")]
#[case("otpauth-migration://offline?data=AAAA", "not a payload")]
fn garbage_payloads_fail_loudly(#[case] uri: &str, #[case] why: &str) {
    assert!(
        matches!(parse_migration(uri), Err(OtpError::BadMigration(_))),
        "{why} must be BadMigration"
    );
}

#[rstest]
fn a_non_migration_uri_is_not_otpauth() {
    assert!(matches!(
        parse_migration("otpauth://totp/x?secret=GEZDGNBV"),
        Err(OtpError::NotOtpauth(_))
    ));
}
