//! The import front door: sniffing pasted text, decoded QR payloads and
//! export files into reviewable account candidates.

use castellan_otp::{OtpError, import::preview};
use rstest::rstest;

const SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

#[rstest]
fn a_pasted_otpauth_uri_previews_verbatim() {
    let uri = format!("otpauth://totp/GitHub:octocat?secret={SECRET}&issuer=GitHub");
    let accounts = preview(&uri).unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].issuer.as_deref(), Some("GitHub"));
    assert_eq!(accounts[0].account, "octocat");
    // What the user scanned is what gets stored — no canonicalization
    // that could lose a parameter this build does not know yet.
    assert_eq!(accounts[0].otpauth.as_deref(), Some(uri.as_str()));
}

#[rstest]
fn several_pasted_uris_become_one_batch() {
    let payload = format!(
        "otpauth://totp/A:a?secret={SECRET}\n\notpauth://totp/B:b?secret={SECRET}&algorithm=SHA256\n"
    );
    let accounts = preview(&payload).unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].problem, None);
    // The refused line is present and named, not dropped.
    assert!(accounts[1].otpauth.is_none());
    assert!(accounts[1].problem.is_some());
}

#[rstest]
fn a_migration_uri_routes_to_the_migration_parser() {
    let accounts = preview(
        "otpauth-migration://offline?data=CjEKCkhlbGxvId6tvu8SGEV4YW1wbGU6YWxpY2VAZ29vZ2xlLmNvbRoHRXhhbXBsZSABKAEwAg%3D%3D",
    )
    .unwrap();
    assert_eq!(accounts[0].issuer.as_deref(), Some("Example"));
    assert_eq!(accounts[0].problem, None);
}

#[rstest]
fn an_aegis_plain_export_imports_with_policy_applied() {
    let export = format!(
        r#"{{
          "version": 1,
          "header": {{ "slots": null, "params": null }},
          "db": {{
            "version": 3,
            "entries": [
              {{ "type": "totp", "uuid": "1", "name": "octocat", "issuer": "GitHub",
                 "note": "", "info": {{ "secret": "{SECRET}", "algo": "SHA1", "digits": 6, "period": 30 }} }},
              {{ "type": "totp", "uuid": "2", "name": "alice", "issuer": "Bank",
                 "note": "", "info": {{ "secret": "{SECRET}", "algo": "SHA256", "digits": 6, "period": 30 }} }},
              {{ "type": "steam", "uuid": "3", "name": "gamer", "issuer": "Steam",
                 "note": "", "info": {{ "secret": "{SECRET}", "algo": "SHA1", "digits": 5, "period": 30 }} }}
            ]
          }}
        }}"#
    );
    let accounts = preview(&export).unwrap();
    assert_eq!(accounts.len(), 3);

    assert_eq!(accounts[0].issuer.as_deref(), Some("GitHub"));
    let uri = accounts[0].otpauth.as_deref().unwrap();
    assert!(castellan_otp::parse(uri).is_ok());

    assert!(accounts[1].problem.as_deref().unwrap().contains("SHA-1"));
    assert!(accounts[2].problem.as_deref().unwrap().contains("STEAM"));
}

#[rstest]
fn an_aegis_nondefault_period_survives_import() {
    let export = format!(
        r#"{{ "db": {{ "entries": [
          {{ "type": "totp", "name": "a", "issuer": "I",
             "info": {{ "secret": "{SECRET}", "algo": "sha1", "digits": 8, "period": 60 }} }}
        ] }} }}"#
    );
    let accounts = preview(&export).unwrap();
    let config = castellan_otp::parse(accounts[0].otpauth.as_deref().unwrap()).unwrap();
    assert_eq!(config.digits, 8);
    assert_eq!(config.period, 60);
}

#[rstest]
fn an_encrypted_aegis_export_is_refused_by_name() {
    let export = r#"{
      "version": 1,
      "header": { "slots": [ { "type": 1 } ], "params": { "nonce": "00", "tag": "00" } },
      "db": "ZW5jcnlwdGVkIGJsb2I="
    }"#;
    assert!(matches!(
        preview(export),
        Err(OtpError::EncryptedExport(tool)) if tool == "Aegis"
    ));
}

#[rstest]
fn an_andotp_plain_export_imports() {
    let export = format!(
        r#"[
          {{ "secret": "{SECRET}", "issuer": "GitHub", "label": "octocat",
             "digits": 6, "type": "TOTP", "algorithm": "SHA1", "thumbnail": "Default",
             "last_used": 0, "used_frequency": 0, "period": 30, "tags": [] }},
          {{ "secret": "{SECRET}", "issuer": "", "label": "Legacy:bob",
             "digits": 6, "type": "HOTP", "algorithm": "SHA1", "counter": 3,
             "thumbnail": "Default", "last_used": 0, "used_frequency": 0, "tags": [] }}
        ]"#
    );
    let accounts = preview(&export).unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].issuer.as_deref(), Some("GitHub"));
    assert_eq!(accounts[0].problem, None);
    assert!(accounts[1].problem.as_deref().unwrap().contains("HOTP"));
}

#[rstest]
#[case("", "empty")]
#[case("   \n ", "whitespace")]
#[case("hello world", "prose")]
#[case("{\"some\": \"json\"}", "unrelated object")]
#[case("[1, 2, 3]", "unrelated array")]
#[case("[{\"name\": \"x\"}]", "array without andOTP fields")]
fn unrecognized_payloads_say_so(#[case] payload: &str, #[case] why: &str) {
    assert!(
        matches!(preview(payload), Err(OtpError::UnrecognizedImport)),
        "{why} must be UnrecognizedImport"
    );
}
