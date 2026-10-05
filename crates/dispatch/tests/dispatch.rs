//! Public dispatcher behavior at the transport-independent boundary —
//! session-aware since task-7: entries come from the one [`VaultSession`],
//! locked vaults answer `vault_locked`, and `lock_database` locks the
//! session every face shares.
//!
//! rstest `#[case]` matrices for the per-operation facts (one row per arm
//! of the dispatcher's match, so an operation added to the contract without
//! a dispatch arm fails here first) and proptest for the invariants that
//! hold for *any* input: the correlation id survives any request, and any
//! word count the wire can request comes back as exactly that many words.

use std::path::PathBuf;
use std::sync::Arc;

use castellan_dispatch::Dispatcher;
use castellan_protocol::{NewEntry, RpcErrorCode, RpcMethod, RpcRequest, RpcResult, VaultStatus};
use castellan_vault::VaultSession;
use keepass::{Database, DatabaseKey};
use proptest::prelude::*;
use rstest::{fixture, rstest};

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

/// An `ImportOtpAccounts` selection shaped like a reviewed account —
/// case data, same reasoning as [`sample_entry`].
fn sample_selection() -> castellan_protocol::OtpImportSelection {
    castellan_protocol::OtpImportSelection {
        title: "GitHub".into(),
        username: Some("octocat".into()),
        otpauth: "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP&issuer=GitHub".into(),
    }
}

/// A tiny real vault in a temp file, built through keepass's own builder
/// and opened through the vault crate's front door — the same shape as the
/// vault crate's fixture, one entry so assertions read at a glance. The
/// path is unique per call: tests run concurrently and a shared path would
/// have one fixture's save truncating another's open.
fn vault_file(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut database = Database::new();
    {
        let mut root = database.root_mut();
        let mut entry = root.add_entry();
        entry.set_unprotected("Title", format!("entry-{tag}"));
        entry.set_unprotected("UserName", "octocat");
    }
    let path = std::env::temp_dir().join(format!(
        "castellan-dispatch-tests-{}-{tag}-{unique}.kdbx",
        std::process::id()
    ));
    let mut file = std::fs::File::create(&path).unwrap();
    database
        .save(
            &mut file,
            DatabaseKey::new().with_password("fixture password, never a real one"),
        )
        .unwrap();
    drop(file);
    path
}

/// A dispatcher over a session unlocked on that tiny vault — fresh per
/// test, so a test that locks its session cannot leak into the next.
#[fixture]
fn unlocked_dispatcher() -> Dispatcher {
    let session = VaultSession::new();
    session
        .unlock(
            &vault_file("unlocked"),
            Some("fixture password, never a real one"),
            None,
        )
        .expect("a database this suite just saved must unlock");
    Dispatcher::new(Arc::new(session))
}

/// A dispatcher whose session has never seen a vault: the state every app
/// boots into.
#[fixture]
fn locked_dispatcher() -> Dispatcher {
    Dispatcher::new(Arc::new(VaultSession::new()))
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
#[case(6, RpcMethod::PreviewOtpImport { payload: "otpauth://totp/A:a?secret=JBSWY3DPEHPK3PXP".into() })]
#[case(8, RpcMethod::ImportOtpAccounts { accounts: vec![] })]
fn every_operation_echoes_the_request_id_and_answers_exactly_one_side(
    unlocked_dispatcher: Dispatcher,
    #[case] id: u64,
    #[case] method: RpcMethod,
) {
    let response = unlocked_dispatcher.dispatch(RpcRequest { id, method });

    assert_eq!(response.id, id);
    assert!(
        response.result.is_some() != response.error.is_some(),
        "a response must carry a result or an error, never both and never neither"
    );
}

/// The result variant is the operation's own — the correlation the
/// generated face clients' narrowing depends on. Entries come from the
/// session now (task-7): the tiny fixture vault's one entry is the
/// honest answer, not a scaffold's empty list.
#[rstest]
#[case(RpcMethod::Ping {})]
#[case(RpcMethod::GetEntries { origin: String::new() })]
#[case(RpcMethod::LockDatabase {})]
fn the_result_variant_is_the_operations_own(
    unlocked_dispatcher: Dispatcher,
    #[case] method: RpcMethod,
) {
    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 1,
        method: method.clone(),
    });

    let matched = match (&method, &response.result) {
        (RpcMethod::Ping {}, Some(RpcResult::Ping {})) => true,
        (RpcMethod::GetEntries { .. }, Some(RpcResult::GetEntries { entries })) => {
            entries.len() == 1 && entries[0].title == "entry-unlocked"
        }
        (RpcMethod::LockDatabase {}, Some(RpcResult::LockDatabase {})) => true,
        _ => false,
    };
    assert!(matched, "result variant must correlate with {method:?}");
    assert!(response.error.is_none());
}

/// The operation this build still refuses answers inside the envelope,
/// with the stable code the app's error mapping branches on — an
/// application failure is a response, not a broken transport.
#[rstest]
#[case(RpcMethod::SaveEntry { entry: sample_entry() })]
fn unimplemented_operations_fail_inside_the_envelope(
    unlocked_dispatcher: Dispatcher,
    #[case] method: RpcMethod,
) {
    let response = unlocked_dispatcher.dispatch(RpcRequest { id: 9, method });

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
fn generated_passphrases_have_the_requested_shape(
    unlocked_dispatcher: Dispatcher,
    #[case] words: u32,
    #[case] separator: &str,
) {
    let response = unlocked_dispatcher.dispatch(RpcRequest {
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
    unlocked_dispatcher: Dispatcher,
    #[case] requested: u32,
    #[case] expected: usize,
) {
    let response = unlocked_dispatcher.dispatch(RpcRequest {
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

// ── The session the dispatcher answers from (task-7) ─────────────────────────

/// A locked vault answers list requests with the stable code — not a
/// transport failure, not a scaffold's empty list — and "locked" outranks
/// "not implemented" for the operations this build cannot run yet: the
/// fill UI's next screen depends on which it hears.
#[rstest]
#[case(RpcMethod::GetEntries { origin: "https://github.com".into() })]
#[case(RpcMethod::GetTotp { entry_id: "entry-id".into() })]
#[case(RpcMethod::SaveEntry { entry: sample_entry() })]
#[case(RpcMethod::ImportOtpAccounts { accounts: vec![sample_selection()] })]
fn a_locked_vault_answers_with_vault_locked(
    locked_dispatcher: Dispatcher,
    #[case] method: RpcMethod,
) {
    let response = locked_dispatcher.dispatch(RpcRequest { id: 4, method });

    assert_eq!(response.id, 4);
    assert!(response.result.is_none());
    assert_eq!(
        response.error.expect("locked sets the error").code,
        RpcErrorCode::VaultLocked
    );
}

/// `lock_database` locks the session the faces share: the response says
/// ok, the status flips, and subscribers hear the event the UI listens
/// for — one call, every face in sync.
#[rstest]
fn lock_database_locks_the_shared_session(unlocked_dispatcher: Dispatcher) {
    let session = unlocked_dispatcher.session();
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorder = {
        let seen = Arc::clone(&seen);
        Arc::new(move |event: &castellan_protocol::Event| {
            seen.lock().expect("events").push(event.clone())
        })
    };
    session.subscribe(recorder);

    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 2,
        method: RpcMethod::LockDatabase {},
    });

    assert!(matches!(response.result, Some(RpcResult::LockDatabase {})));
    assert_eq!(session.status(), VaultStatus::Locked);
    assert_eq!(
        *seen.lock().expect("events"),
        vec![castellan_protocol::Event::DatabaseLocked]
    );
    // And the next list request now answers locked.
    let after = unlocked_dispatcher.dispatch(RpcRequest {
        id: 3,
        method: RpcMethod::GetEntries {
            origin: String::new(),
        },
    });
    assert_eq!(
        after.error.expect("locked after lock").code,
        RpcErrorCode::VaultLocked
    );
}

/// A session unlocked through the vault crate's front door is the same
/// session the dispatcher answers from: entries a save-then-open built are
/// the entries the faces list.
#[rstest]
fn unlock_flows_through_to_the_faces(unlocked_dispatcher: Dispatcher) {
    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 8,
        method: RpcMethod::GetEntries {
            origin: "https://github.com".into(),
        },
    });

    let Some(RpcResult::GetEntries { entries }) = response.result else {
        panic!("get_entries returned the wrong result variant");
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "entry-unlocked");
    assert_eq!(entries[0].username.as_deref(), Some("octocat"));
}

// ── TOTP: codes and the import pipeline (task-13) ────────────────────────────

/// A dispatcher whose vault carries one TOTP entry — the seed is the
/// classic test secret, so the expected code is plain arithmetic.
#[fixture]
fn totp_dispatcher() -> Dispatcher {
    let path = vault_file("totp");
    let session = VaultSession::new();
    session
        .unlock(&path, Some("fixture password, never a real one"), None)
        .expect("a database this suite just saved must unlock");
    let dispatcher = Dispatcher::new(Arc::new(session));
    let response = dispatcher.dispatch(RpcRequest {
        id: 1,
        method: RpcMethod::ImportOtpAccounts {
            accounts: vec![sample_selection()],
        },
    });
    assert!(response.error.is_none(), "fixture import must succeed");
    dispatcher
}

/// The id of the fixture's TOTP entry, read the way a face would: from
/// the entry list, by its badge.
fn totp_entry_id(dispatcher: &Dispatcher) -> String {
    let response = dispatcher.dispatch(RpcRequest {
        id: 2,
        method: RpcMethod::GetEntries {
            origin: "https://github.com".into(),
        },
    });
    let Some(RpcResult::GetEntries { entries }) = response.result else {
        panic!("entries must answer");
    };
    entries
        .into_iter()
        .find(|entry| entry.has_totp)
        .expect("the imported TOTP entry is in the list")
        .id
}

/// `get_totp` answers the code the stored seed produces *right now* —
/// verified against the otp crate directly, same seed, same instant.
/// The window guard keeps the test honest across a period boundary.
#[rstest]
fn get_totp_computes_from_the_stored_seed(totp_dispatcher: Dispatcher) {
    let entry_id = totp_entry_id(&totp_dispatcher);
    let config = castellan_otp::parse(&sample_selection().otpauth).unwrap();

    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let response = totp_dispatcher.dispatch(RpcRequest {
        id: 3,
        method: RpcMethod::GetTotp { entry_id },
    });
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let Some(RpcResult::GetTotp {
        code,
        seconds_remaining,
    }) = response.result
    else {
        panic!("get_totp must answer its own variant: {response:?}");
    };
    let (expected_before, _) = castellan_otp::code_now(&config, before);
    let (expected_after, _) = castellan_otp::code_now(&config, after);
    assert!(
        code == expected_before || code == expected_after,
        "the code must be the seed's code for the dispatch instant"
    );
    assert!((1..=30).contains(&seconds_remaining));
}

/// The two no-answer shapes stay distinguishable for the UI: a row that
/// was never there vs a row whose entry carries no seed — both the
/// stable `no_such_entry` code, each with its own message.
#[rstest]
fn get_totp_names_missing_entries_and_missing_seeds(unlocked_dispatcher: Dispatcher) {
    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 4,
        method: RpcMethod::GetTotp {
            entry_id: "00000000-0000-0000-0000-000000000000".into(),
        },
    });
    let error = response.error.expect("a missing entry is an error");
    assert_eq!(error.code, RpcErrorCode::NoSuchEntry);

    // The fixture vault's one entry exists but has no seed.
    let entries = match unlocked_dispatcher
        .dispatch(RpcRequest {
            id: 5,
            method: RpcMethod::GetEntries {
                origin: String::new(),
            },
        })
        .result
    {
        Some(RpcResult::GetEntries { entries }) => entries,
        other => panic!("entries must answer: {other:?}"),
    };
    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 6,
        method: RpcMethod::GetTotp {
            entry_id: entries[0].id.clone(),
        },
    });
    let error = response.error.expect("a seedless entry is an error");
    assert_eq!(error.code, RpcErrorCode::NoSuchEntry);
    assert!(error.message.contains("no TOTP seed"));
}

/// Preview is pure parsing and deliberately answers while locked — a
/// scan is reviewed wherever it happens, and nothing is stored.
#[rstest]
fn preview_otp_import_answers_even_locked(locked_dispatcher: Dispatcher) {
    let response = locked_dispatcher.dispatch(RpcRequest {
        id: 7,
        method: RpcMethod::PreviewOtpImport {
            payload: "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP&issuer=GitHub".into(),
        },
    });
    let Some(RpcResult::PreviewOtpImport { accounts }) = response.result else {
        panic!("preview must answer its own variant: {response:?}");
    };
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].issuer.as_deref(), Some("GitHub"));
    assert!(accounts[0].otpauth.is_some());
    assert!(accounts[0].problem.is_none());
}

/// An unparseable payload is the caller's input problem: `bad_request`,
/// not `not_implemented` — the UI's "fix what you pasted" branch.
#[rstest]
fn preview_of_garbage_is_bad_request(locked_dispatcher: Dispatcher) {
    let response = locked_dispatcher.dispatch(RpcRequest {
        id: 8,
        method: RpcMethod::PreviewOtpImport {
            payload: "certainly not an export".into(),
        },
    });
    assert_eq!(
        response.error.expect("garbage is an error").code,
        RpcErrorCode::BadRequest
    );
}

/// A selection with one unusable seed rejects the whole batch before
/// anything is stored — the review's checkboxes are the unit of consent,
/// and a half-imported batch is the state they exist to prevent.
#[rstest]
fn import_with_a_bad_seed_rejects_the_whole_batch(unlocked_dispatcher: Dispatcher) {
    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 9,
        method: RpcMethod::ImportOtpAccounts {
            accounts: vec![
                sample_selection(),
                castellan_protocol::OtpImportSelection {
                    title: "Broken".into(),
                    username: None,
                    otpauth: "otpauth://totp/Broken:x".into(), // no secret
                },
            ],
        },
    });
    assert_eq!(
        response.error.expect("a bad seed is an error").code,
        RpcErrorCode::BadRequest
    );

    // Nothing landed: the vault still has only the fixture's entry.
    let entries = match unlocked_dispatcher
        .dispatch(RpcRequest {
            id: 10,
            method: RpcMethod::GetEntries {
                origin: String::new(),
            },
        })
        .result
    {
        Some(RpcResult::GetEntries { entries }) => entries,
        other => panic!("entries must answer: {other:?}"),
    };
    assert_eq!(entries.len(), 1);
}

/// The import round trip a face sees: import answers the count, the
/// session announces each entry, and the new entry serves codes.
#[rstest]
fn imported_accounts_show_up_and_serve_codes(unlocked_dispatcher: Dispatcher) {
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorder = {
        let seen = Arc::clone(&seen);
        Arc::new(move |event: &castellan_protocol::Event| {
            seen.lock().expect("events").push(event.clone())
        })
    };
    unlocked_dispatcher.session().subscribe(recorder);

    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 11,
        method: RpcMethod::ImportOtpAccounts {
            accounts: vec![sample_selection()],
        },
    });
    let Some(RpcResult::ImportOtpAccounts { imported }) = response.result else {
        panic!("import must answer its own variant: {response:?}");
    };
    assert_eq!(imported, 1);

    let events = seen.lock().expect("events");
    assert!(
        events
            .iter()
            .any(|event| matches!(event, castellan_protocol::Event::EntryChanged { .. })),
        "faces refresh on entry_changed; the import must announce"
    );
    drop(events);

    let entry_id = totp_entry_id(&unlocked_dispatcher);
    let response = unlocked_dispatcher.dispatch(RpcRequest {
        id: 12,
        method: RpcMethod::GetTotp { entry_id },
    });
    assert!(response.result.is_some(), "the new entry must serve codes");
}

/// Any operation the contract defines, with arbitrary field values — so
/// the properties below hold over the whole request space, not the few
/// shapes a hand-written test thought to try. The session stays unlocked
/// throughout: `LockDatabase` and the locked-error precedence are covered
/// by the deterministic tests above.
fn any_method() -> impl Strategy<Value = RpcMethod> {
    prop_oneof![
        Just(RpcMethod::Ping {}),
        (proptest::collection::vec(proptest::char::any(), 0..32)).prop_map(|origin| {
            RpcMethod::GetEntries {
                origin: origin.into_iter().collect(),
            }
        }),
        (proptest::collection::vec(proptest::char::any(), 0..32)).prop_map(|entry_id| {
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
        // Preview parses arbitrary text; the property run proves the
        // sniffing never panics and always stays inside the envelope.
        (proptest::collection::vec(proptest::char::any(), 0..64)).prop_map(|payload| {
            RpcMethod::PreviewOtpImport {
                payload: payload.into_iter().collect(),
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
        let dispatcher = Dispatcher::new(Arc::new(VaultSession::new()));
        let response = dispatcher.dispatch(RpcRequest { id, method });
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
        let dispatcher = Dispatcher::new(Arc::new(VaultSession::new()));
        let response = dispatcher.dispatch(RpcRequest {
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
