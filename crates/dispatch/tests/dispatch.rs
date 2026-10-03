//! Public dispatcher behavior at the transport-independent boundary.
//!
//! rstest `#[case]` matrices for the per-operation facts (one row per arm
//! of the dispatcher's match, so an operation added to the contract without
//! a dispatch arm fails here first) and proptest for the invariants that
//! hold for *any* input: the correlation id survives any request, and any
//! word count the wire can request comes back as exactly that many words.

use castellan_dispatch::dispatch;
use castellan_protocol::{NewEntry, RpcErrorCode, RpcMethod, RpcRequest, RpcResult};
use proptest::prelude::*;
use rstest::rstest;

/// A `SaveEntry` payload shaped like a real capture. A plain builder, not a
/// `#[fixture]`: it is case data evaluated per row, not a setup a test
/// would otherwise repeat.
fn sample_entry() -> NewEntry {
    NewEntry {
        title: "GitHub".into(),
        username: Some("octocat".into()),
        password: Some("fixture password, never a real one".into()),
        url: Some("https://github.com".into()),
        otpauth: Some("otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP".into()),
    }
}

/// One row per operation in the contract. Whichever face sent it and
/// whichever arm answers it, the response echoes the request id and
/// answers on exactly one side of the envelope — never both, never
/// neither. The id half is what every face's pending-request map is
/// keyed on; the exactly-one half is what `RpcClient`'s narrowing
/// (`result !== null` xor `error !== null`) assumes.
#[rstest]
#[case(42, RpcMethod::Ping {})]
#[case(7, RpcMethod::GetEntries { origin: "https://github.com".into() })]
#[case(3, RpcMethod::GeneratePassphrase { words: 4, separator: "-".into() })]
#[case(11, RpcMethod::LockDatabase {})]
#[case(9, RpcMethod::GetTotp { entry_id: "entry-id".into() })]
#[case(5, RpcMethod::SaveEntry { entry: sample_entry() })]
fn every_operation_echoes_the_request_id_and_answers_exactly_one_side(
    #[case] id: u64,
    #[case] method: RpcMethod,
) {
    let response = dispatch(RpcRequest { id, method });

    assert_eq!(response.id, id);
    assert!(
        response.result.is_some() != response.error.is_some(),
        "a response must carry a result or an error, never both and never neither"
    );
}

/// The result variant is the operation's own — the correlation the
/// generated face clients' narrowing depends on. (Passphrase answers with
/// a random value, so its row proves shape, not identity, in the property
/// below; the two unimplemented operations prove their error rows in the
/// next test.)
#[rstest]
#[case(RpcMethod::Ping {})]
#[case(RpcMethod::GetEntries { origin: String::new() })]
#[case(RpcMethod::LockDatabase {})]
fn the_result_variant_is_the_operations_own(#[case] method: RpcMethod) {
    let response = dispatch(RpcRequest {
        id: 1,
        method: method.clone(),
    });

    let matched = match (&method, &response.result) {
        (RpcMethod::Ping {}, Some(RpcResult::Ping {})) => true,
        (RpcMethod::GetEntries { .. }, Some(RpcResult::GetEntries { entries })) => {
            // No open-vault session yet: the honest answer is empty, not
            // an error — the fill UI renders "nothing for this origin".
            entries.is_empty()
        }
        (RpcMethod::LockDatabase {}, Some(RpcResult::LockDatabase {})) => true,
        _ => false,
    };
    assert!(matched, "result variant must correlate with {method:?}");
    assert!(response.error.is_none());
}

/// The two operations this build refuses still answer inside the
/// envelope, with the stable code the app's error mapping branches on —
/// an application failure is a response, not a broken transport.
#[rstest]
#[case(RpcMethod::GetTotp { entry_id: "entry-id".into() })]
#[case(RpcMethod::SaveEntry { entry: sample_entry() })]
fn unimplemented_operations_fail_inside_the_envelope(#[case] method: RpcMethod) {
    let response = dispatch(RpcRequest { id: 9, method });

    assert_eq!(response.id, 9);
    assert!(response.result.is_none());
    assert_eq!(
        response
            .error
            .expect("the unimplemented arm sets the error")
            .code,
        RpcErrorCode::NotImplemented
    );
}

/// The generation contract the passphrase UI relies on: ask for N words,
/// get N joined words — through the dispatcher, the way every native face
/// actually asks.
#[rstest]
#[case(4, "-")]
#[case(1, ".")]
#[case(16, "_")]
fn generated_passphrases_have_the_requested_shape(#[case] words: u32, #[case] separator: &str) {
    let response = dispatch(RpcRequest {
        id: 7,
        method: RpcMethod::GeneratePassphrase {
            words,
            separator: separator.into(),
        },
    });

    let Some(RpcResult::GeneratePassphrase { value }) = response.result else {
        panic!("generate_passphrase returned the wrong result variant");
    };
    assert_eq!(value.split(separator).count(), words as usize);
    assert!(response.error.is_none());
}

/// The dispatcher clamps word counts into the 1..=16 range the vault's
/// generator is sized for, so a hostile or buggy caller cannot ask the
/// passphrase fn for zero words (an empty string the UI would render as a
/// generated password) or a thousand (a denial of patience).
#[rstest]
#[case(0, 1)]
#[case(17, 16)]
#[case(u32::MAX, 16)]
fn out_of_range_word_counts_clamp_into_the_contract(
    #[case] requested: u32,
    #[case] expected: usize,
) {
    let response = dispatch(RpcRequest {
        id: 1,
        method: RpcMethod::GeneratePassphrase {
            words: requested,
            separator: "-".into(),
        },
    });

    let Some(RpcResult::GeneratePassphrase { value }) = response.result else {
        panic!("generate_passphrase returned the wrong result variant");
    };
    assert_eq!(value.split('-').count(), expected);
}

/// Any operation the contract defines, with arbitrary field values — so
/// the properties below hold over the whole request space, not the few
/// shapes a hand-written test thought to try.
fn any_method() -> impl Strategy<Value = RpcMethod> {
    prop_oneof![
        Just(RpcMethod::Ping {}),
        Just(RpcMethod::LockDatabase {}),
        proptest::collection::vec(proptest::char::any(), 0..32).prop_map(|origin| {
            RpcMethod::GetEntries {
                origin: origin.into_iter().collect(),
            }
        }),
        proptest::collection::vec(proptest::char::any(), 0..32).prop_map(|entry_id| {
            RpcMethod::GetTotp {
                entry_id: entry_id.into_iter().collect(),
            }
        }),
        (1u32..=16, proptest::char::any()).prop_map(|(words, separator)| {
            RpcMethod::GeneratePassphrase {
                words,
                separator: separator.to_string(),
            }
        }),
    ]
}

proptest! {
    /// For any id and any operation, the response carries that id back —
    /// the correlation every face's pending-request map depends on, and
    /// the one thing a dispatcher must never drop or rewrite.
    #[test]
    fn any_request_id_survives_the_dispatcher(
        id in any::<u64>(),
        method in any_method(),
    ) {
        let response = dispatch(RpcRequest { id, method });
        prop_assert_eq!(response.id, id);
    }

    /// For any in-range word count and any separator, the dispatched
    /// passphrase is exactly that many words joined by that separator —
    /// the contract the passphrase editor's preview shows, over the whole
    /// domain it can request. Wordlist membership is the vault crate's
    /// own property (passphrase.rs), not the dispatcher's to re-prove.
    #[test]
    fn passphrase_shape_holds_for_any_request(
        words in 1u32..=16,
        separator in prop_oneof![Just("-"), Just("."), Just("_"), Just(" ")],
    ) {
        let response = dispatch(RpcRequest {
            id: 1,
            method: RpcMethod::GeneratePassphrase { words, separator: separator.into() },
        });
        let Some(RpcResult::GeneratePassphrase { value }) = response.result else {
            panic!("generate_passphrase returned the wrong result variant");
        };
        let split: Vec<&str> = value.split(separator).collect();
        prop_assert_eq!(split.len(), words as usize);
    }
}
