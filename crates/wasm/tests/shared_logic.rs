//! The WASM face's host-side tests: the same functions the extension
//! runs in-browser, exercised natively. `otpauth_info` and
//! `origin_matches` are plain Rust under the `wasm_bindgen` wrappers, so
//! these tests hit the real logic, not a JS re-implementation.
//!
//! The two `origin_matches` tests below are deliberately shaped like the
//! protocol crate's own suite (matrices for the discrete shapes, a
//! property for the generated ones): this file's job is to prove the
//! *delegation* — that the WASM face answers exactly what the protocol
//! crate answers — because "same scoped logic in every face" is the
//! promise the repo is built around (README).

use castellan_protocol::matching::origin_matches as protocol_origin_matches;
use castellan_wasm::{origin_matches, otpauth_info};
use proptest::prelude::*;
use rstest::rstest;

#[rstest]
// The shapes the fill UI actually meets: bare host, URL, case drift.
#[case("github.com", "https://github.com/login", true)]
#[case("GITHUB.COM", "github.com", true)]
#[case("login.example.org", "example.org", true)]
#[case("example.org", "login.example.org", false)]
#[case("evil.example.org", "login.example.org", false)]
#[case("localhost:3000", "localhost", true)]
fn the_wasm_face_answers_the_documented_matrix(
    #[case] origin: &str,
    #[case] candidate: &str,
    #[case] expected: bool,
) {
    assert_eq!(origin_matches(origin, candidate), expected);
}

/// A lowercase alphabetic label: `a`–`z`, 1–11 chars — the same strategy
/// the protocol crate's suite uses, so the agreement property below
/// compares the two faces over one input space, not two accidents.
fn label() -> impl Strategy<Value = String> {
    (any::<u8>(), prop::collection::vec(any::<u8>(), 0..10)).prop_map(|(first, rest)| {
        std::iter::once(first)
            .chain(rest)
            .map(|b| char::from(b'a' + (b % 26)))
            .collect()
    })
}

/// A 2–4 label host: `example.org` shape.
fn host() -> impl Strategy<Value = String> {
    (label(), label(), prop::collection::vec(label(), 0..2)).prop_map(|(first, second, more)| {
        std::iter::once(first)
            .chain(std::iter::once(second))
            .chain(more)
            .collect::<Vec<_>>()
            .join(".")
    })
}

proptest! {
    /// The delegation invariant, for any host the matching rule is defined
    /// over: the WASM re-export and the protocol crate answer identically.
    /// A drift here is two halves of the product disagreeing about "does
    /// this entry fill here?" — the failure mode this repo's structure
    /// exists to make impossible.
    #[test]
    fn the_wasm_face_never_disagrees_with_the_protocol_crate(
        origin in host(),
        candidate in host(),
    ) {
        prop_assert_eq!(
            origin_matches(&origin, &candidate),
            protocol_origin_matches(&origin, &candidate),
        );
    }
}

#[rstest]
// The wrapper's own surface: whatever shape the URI arrives in, the info
// comes back parsed and the secret never does — the extension's fill UI
// reads this struct, and a secret field here is a secret on a page.
#[case(
    "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP",
    Some("GitHub"),
    "octocat"
)]
#[case(
    "otpauth://totp/octocat?secret=JBSWY3DPEHPK3PXP&issuer=Param",
    Some("Param"),
    "octocat"
)]
#[case("otpauth://totp/octocat?secret=JBSWY3DPEHPK3PXP", None, "octocat")]
fn parses_without_exposing_the_secret(
    #[case] uri: &str,
    #[case] issuer: Option<&str>,
    #[case] account: &str,
) {
    let info = otpauth_info(uri).unwrap();
    assert_eq!(info.issuer.as_deref(), issuer);
    assert_eq!(info.account, account);
    assert_eq!(info.digits, 6);
    assert_eq!(info.period, 30);
    // The struct is the whole surface: no secret field exists to leak.
    assert!(!format!("{info:?}").contains("JBSW"));
}
