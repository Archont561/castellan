//! The WASM face's host-side tests: the same functions the extension
//! runs in-browser, exercised natively. `otpauth_info` and
//! `origin_matches` are plain Rust under the `wasm_bindgen` wrappers, so
//! these tests hit the real logic, not a JS re-implementation.

use castellan_wasm::{origin_matches, otpauth_info};

#[test]
fn parses_without_exposing_the_secret() {
    let info = otpauth_info("otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP").unwrap();
    assert_eq!(info.issuer.as_deref(), Some("GitHub"));
    assert_eq!(info.account, "octocat");
    assert_eq!(info.digits, 6);
    assert_eq!(info.period, 30);
    // The struct is the whole surface: no secret field exists to leak.
    assert!(!format!("{info:?}").contains("JBSW"));
}

#[test]
fn matching_agrees_with_the_protocol_crate() {
    assert!(origin_matches("login.example.org", "example.org"));
    assert!(!origin_matches("example.org", "login.example.org"));
}
