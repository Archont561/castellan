//! The unlock/lock session: the one state machine every face's dispatcher
//! calls into (task-7). Time is data here — every policy decision takes its
//! `now` as a parameter, so these tests script time as literal values
//! instead of sleeping, and the apps pass `SystemTime::now()` in production.
//!
//! The corpus cases run against the committed fixtures (see `fixtures/`):
//! real files authored by KeePass clients, not databases this crate built.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use castellan_protocol::{Event, RpcErrorCode, VaultStatus};
use castellan_vault::{
    LockPolicy, LockReason, LockTrigger, SessionClock, VaultError, VaultSession,
};
use rstest::{fixture, rstest};

/// A fixed moment every scripted timeline starts from:
/// 2026-10-03T12:00:00Z.
const T0: SystemTime = SystemTime::UNIX_EPOCH;

/// The scripted clock: the session under test reads this instead of the
/// wall, so lock policy is decided by arithmetic, never by how long the
/// test happened to run. Set to [`T0`] then advanced in steps.
struct ScriptedClock(std::sync::Mutex<SystemTime>);

impl ScriptedClock {
    fn at(start: SystemTime) -> Self {
        Self(std::sync::Mutex::new(start))
    }

    fn advance(&self, by: Duration) {
        let mut now = self.0.lock().expect("scripted clock");
        *now += by;
    }
}

impl SessionClock for ScriptedClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().expect("scripted clock")
    }
}

/// Where the committed corpus lives.
fn corpus(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Records every event the session emits, for asserting what faces would see.
#[derive(Clone, Default)]
struct EventLog {
    seen: Arc<Mutex<Vec<Event>>>,
}

impl EventLog {
    fn new() -> Self {
        Self::default()
    }

    fn sink(&self) -> Arc<dyn Fn(&Event) + Send + Sync> {
        let seen = Arc::clone(&self.seen);
        Arc::new(move |event: &Event| seen.lock().expect("event log").push(event.clone()))
    }

    fn events(&self) -> Vec<Event> {
        self.seen.lock().expect("event log").clone()
    }
}

/// A session unlocked against the Argon2id corpus fixture, ready to be
/// poked — the pytest-fixture pattern, via rstest. Fresh per test: a test
/// that locks its session cannot leak into the next one.
#[fixture]
fn unlocked_session() -> VaultSession {
    let session = VaultSession::new();
    session
        .unlock(
            &corpus("test_db_kdbx4_with_password_argon2id.kdbx"),
            Some("demopass"),
            None,
        )
        .expect("the corpus fixture must unlock");
    session
}

// ── The corpus: real files, real keys (AC-1) ─────────────────────────────────

/// Key material for one corpus fixture: what unlocks it, and what the
/// projection should find inside — the worked example the case matrix
/// asserts against, read off the files themselves.
struct CorpusCase {
    file: &'static str,
    password: Option<&'static str>,
    keyfile: Option<&'static str>,
    expected_titles: &'static [&'static str],
}

const CORPUS: &[CorpusCase] = &[
    CorpusCase {
        file: "test_db_kdbx4_with_password_argon2.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["Test", ""],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_argon2id.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["Test", ""],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_argon2id_chacha20.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["test"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_argon2id_twofish.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["test"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_argon2_chacha20.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["test"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_argon2_twofish.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["test"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_aes.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["ASDF"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_password_deleted_entry.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["Test", "", "deleted entry"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_totp_entry.kdbx",
        password: Some("test"),
        keyfile: None,
        expected_titles: &["this entry has totp"],
    },
    CorpusCase {
        file: "test_db_kdbx41_features.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &["tagged-entry-41", "ayyyyo"],
    },
    CorpusCase {
        file: "test_db_kdbx41_with_password_aes.kdbx",
        password: Some("demopass"),
        keyfile: None,
        expected_titles: &[
            "entry with no quality check",
            "entry with named custom icon",
            "entry that was moved",
            "entry with custom data",
        ],
    },
    CorpusCase {
        file: "test_db_with_password.kdbx",
        password: Some("demopass"),
        keyfile: None,
        // KDBX 3.1 — reading it is part of the compatibility promise.
        expected_titles: &[
            "Sample Entry",
            "",
            "Sample Entry #2",
            "Sample Entry #3",
            "test entry",
            "asdf",
        ],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_keyfile.kdbx",
        password: None,
        keyfile: Some("test_key.key"),
        expected_titles: &["Test"],
    },
    CorpusCase {
        file: "test_db_kdbx4_with_keyfile_v2.kdbx",
        password: Some("demopass"),
        keyfile: Some("test_db_kdbx4_with_keyfile_v2.keyx"),
        expected_titles: &["secret"],
    },
];

#[rstest]
#[case(&CORPUS[0])]
#[case(&CORPUS[1])]
#[case(&CORPUS[2])]
#[case(&CORPUS[3])]
#[case(&CORPUS[4])]
#[case(&CORPUS[5])]
#[case(&CORPUS[6])]
#[case(&CORPUS[7])]
#[case(&CORPUS[8])]
#[case(&CORPUS[9])]
#[case(&CORPUS[10])]
#[case(&CORPUS[11])]
#[case(&CORPUS[12])]
#[case(&CORPUS[13])]
fn unlocks_every_corpus_file_and_projects_its_entries(#[case] case: &CorpusCase) {
    let session = VaultSession::new();
    let keyfile = case.keyfile.map(corpus);

    session
        .unlock(&corpus(case.file), case.password, keyfile.as_deref())
        .expect("a corpus fixture must unlock with its documented key");

    assert_eq!(session.status(), VaultStatus::Unlocked);
    let entries = session
        .entries()
        .expect("an unlocked session lists entries");
    let mut titles: Vec<&str> = entries.iter().map(|entry| entry.title.as_str()).collect();
    titles.sort_unstable();
    let mut expected = case.expected_titles.to_vec();
    expected.sort_unstable();
    assert_eq!(titles, expected, "projected titles for {}", case.file);
}

#[rstest]
#[case(
    "test_db_kdbx4_with_password_argon2id.kdbx",
    "demopass",
    "not the password"
)]
// KDBX 3 fails at decryption rather than the key check — wrong key all
// the same, and the user's question ("is it the password?") is identical.
#[case("test_db_with_password.kdbx", "demopass", "also not the password")]
fn a_wrong_password_fails_with_the_stable_bad_credentials_code(
    #[case] file: &str,
    #[case] good: &str,
    #[case] bad: &str,
) {
    let session = VaultSession::new();

    let error = session
        .unlock(&corpus(file), Some(bad), None)
        .expect_err("a wrong key must not unlock");

    assert!(
        matches!(error, VaultError::Credentials),
        "wrong key must map to Credentials, got {error:?}"
    );
    assert_eq!(error.error_code(), Some(RpcErrorCode::BadCredentials));
    assert_eq!(session.status(), VaultStatus::NoDatabase);
    // And the right key still works afterwards — the failed attempt left
    // no half-open state behind.
    session
        .unlock(&corpus(file), Some(good), None)
        .expect("the correct key must unlock after a failed attempt");
}

#[rstest]
fn a_password_is_not_the_keyfiles_key() {
    let session = VaultSession::new();

    let error = session
        .unlock(
            &corpus("test_db_kdbx4_with_keyfile.kdbx"),
            Some("demopass"),
            None,
        )
        .expect_err("a password cannot open a keyfile-only vault");
    assert_eq!(error.error_code(), Some(RpcErrorCode::BadCredentials));

    session
        .unlock(
            &corpus("test_db_kdbx4_with_keyfile.kdbx"),
            None,
            Some(corpus("test_key.key").as_path()),
        )
        .expect("the keyfile must unlock it");
    assert_eq!(session.status(), VaultStatus::Unlocked);
}

// ── Locking and the wipe (AC-2) ──────────────────────────────────────────────

#[rstest]
fn lock_wipes_the_session_and_answers_with_vault_locked(unlocked_session: VaultSession) {
    assert!(unlocked_session.lock(LockReason::User));

    assert_eq!(unlocked_session.status(), VaultStatus::Locked);
    let error = unlocked_session
        .entries()
        .expect_err("a locked session has no entries to list");
    assert_eq!(error.error_code(), Some(RpcErrorCode::VaultLocked));
    // The UI remembers which file to offer: locked, not amnesiac.
    assert_eq!(
        unlocked_session.last_path(),
        Some(corpus("test_db_kdbx4_with_password_argon2id.kdbx"))
    );
}

#[rstest]
fn locking_an_already_locked_session_is_a_no_op(unlocked_session: VaultSession) {
    unlocked_session.lock(LockReason::User);

    let log = EventLog::new();
    unlocked_session.subscribe(log.sink());
    assert!(!unlocked_session.lock(LockReason::User));

    assert!(log.events().is_empty(), "no state change, no event");
}

#[rstest]
fn a_fresh_session_is_empty_and_answers_like_a_locked_one() {
    let session = VaultSession::new();

    assert_eq!(session.status(), VaultStatus::NoDatabase);
    assert_eq!(session.last_path(), None);
    assert_eq!(
        session.entries().expect_err("no vault").error_code(),
        Some(RpcErrorCode::VaultLocked)
    );
}

// ── The event bus (AC-4) ─────────────────────────────────────────────────────

#[rstest]
fn unlock_and_lock_fire_the_status_events_faces_listen_for() {
    let session = VaultSession::new();
    let log = EventLog::new();
    session.subscribe(log.sink());

    session
        .unlock(
            &corpus("test_db_kdbx4_with_password_argon2id.kdbx"),
            Some("demopass"),
            None,
        )
        .expect("the corpus fixture must unlock");
    session.lock(LockReason::User);

    assert_eq!(
        log.events(),
        vec![Event::DatabaseUnlocked, Event::DatabaseLocked]
    );
}

#[rstest]
fn a_failed_unlock_fires_nothing() {
    let session = VaultSession::new();
    let log = EventLog::new();
    session.subscribe(log.sink());

    let _ = session.unlock(
        &corpus("test_db_kdbx4_with_password_argon2id.kdbx"),
        Some("wrong"),
        None,
    );

    assert!(log.events().is_empty());
}

#[rstest]
fn subscribers_can_leave_and_stop_hearing(unlocked_session: VaultSession) {
    let staying = EventLog::new();
    let leaving = EventLog::new();
    let id = unlocked_session.subscribe(leaving.sink());
    unlocked_session.subscribe(staying.sink());

    unlocked_session.unsubscribe(id);
    unlocked_session.lock(LockReason::User);

    assert!(leaving.events().is_empty());
    assert_eq!(staying.events(), vec![Event::DatabaseLocked]);
}

#[rstest]
fn unlocking_over_an_open_session_replaces_it_and_both_events_fire(unlocked_session: VaultSession) {
    let log = EventLog::new();
    unlocked_session.subscribe(log.sink());

    unlocked_session
        .unlock(
            &corpus("test_db_kdbx4_with_password_aes.kdbx"),
            Some("demopass"),
            None,
        )
        .expect("the second corpus fixture must unlock");

    // The old vault locks (its key material drops), then the new one opens.
    assert_eq!(
        log.events(),
        vec![Event::DatabaseLocked, Event::DatabaseUnlocked]
    );
    assert_eq!(
        unlocked_session.last_path(),
        Some(corpus("test_db_kdbx4_with_password_aes.kdbx"))
    );
}

// ── Soft lock tiers (AC-3): blur / wake / idle, configured by policy ─────────

/// The default policy the product ships: blur grace 5 min, lock on wake,
/// idle 15 min, hard after 7 days.
fn default_policy() -> LockPolicy {
    LockPolicy::default()
}

/// A session unlocked at [`T0`] on a scripted clock, so every policy test
/// advances time explicitly. Fresh per test: the clock and the session are
/// one per test, and nothing leaks.
#[fixture]
fn timed_session() -> (VaultSession, std::sync::Arc<ScriptedClock>) {
    let clock = std::sync::Arc::new(ScriptedClock::at(T0));
    let session =
        VaultSession::with_clock(std::sync::Arc::clone(&clock) as std::sync::Arc<dyn SessionClock>);
    session
        .unlock(
            &corpus("test_db_kdbx4_with_password_argon2id.kdbx"),
            Some("demopass"),
            None,
        )
        .expect("the corpus fixture must unlock");
    (session, clock)
}

#[rstest]
fn a_new_session_has_no_activity_to_lock_on() {
    let session = VaultSession::new();

    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Idle),
        None
    );
}

#[rstest]
fn blur_within_its_grace_window_does_not_lock(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, clock) = timed_session;
    // The blur lands, and the first minutes of it are the user alt-tabbing
    // back and forth — the grace window exists exactly for them.
    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Blur),
        None
    );
    clock.advance(Duration::from_secs(4 * 60));
    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Blur),
        None
    );
    assert_eq!(session.status(), VaultStatus::Unlocked);
}

#[rstest]
fn blur_beyond_its_grace_window_locks(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, clock) = timed_session;
    session.auto_lock(&default_policy(), LockTrigger::Blur);

    clock.advance(Duration::from_secs(5 * 60));
    let reason = session.auto_lock(&default_policy(), LockTrigger::Blur);

    assert_eq!(reason, Some(LockReason::Blur));
    assert_eq!(session.status(), VaultStatus::Locked);
}

#[rstest]
fn wake_locks_immediately_under_the_default_policy(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, _clock) = timed_session;
    let reason = session.auto_lock(&default_policy(), LockTrigger::Wake);

    assert_eq!(reason, Some(LockReason::Wake));
    assert_eq!(session.status(), VaultStatus::Locked);
}

#[rstest]
fn a_disabled_trigger_never_locks(timed_session: (VaultSession, std::sync::Arc<ScriptedClock>)) {
    let (session, clock) = timed_session;
    let policy = LockPolicy {
        after_blur: None,
        after_wake: None,
        after_idle: None,
        hard_after: None,
    };
    clock.advance(Duration::from_secs(90 * 86_400));

    assert_eq!(session.auto_lock(&policy, LockTrigger::Blur), None);
    assert_eq!(session.auto_lock(&policy, LockTrigger::Wake), None);
    assert_eq!(session.auto_lock(&policy, LockTrigger::Idle), None);
    assert_eq!(session.hard_lock_if_due(&policy), None);
    assert_eq!(session.status(), VaultStatus::Unlocked);
}

#[rstest]
#[case(Duration::from_secs(15 * 60 - 1), None)]
#[case(Duration::from_secs(15 * 60), Some(LockReason::Idle))]
#[case(Duration::from_secs(3 * 60 * 60), Some(LockReason::Idle))]
fn idle_locks_at_the_configured_boundary(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
    #[case] elapsed: Duration,
    #[case] expected: Option<LockReason>,
) {
    let (session, clock) = timed_session;
    clock.advance(elapsed);

    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Idle),
        expected
    );
    assert_eq!(
        session.status(),
        if expected.is_some() {
            VaultStatus::Locked
        } else {
            VaultStatus::Unlocked
        }
    );
}

#[rstest]
fn activity_resets_the_idle_clock_and_cancels_the_blur_grace(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, clock) = timed_session;
    session.auto_lock(&default_policy(), LockTrigger::Blur);

    // The user comes back ten minutes into the blur grace, then idles.
    clock.advance(Duration::from_secs(10 * 60));
    session.touch();

    // The blur grace is gone: the window measured from the original blur
    // would be over, but the session saw activity after it.
    clock.advance(Duration::from_secs(60));
    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Blur),
        None
    );
    // And the idle clock counts from the touch, not from the unlock.
    clock.advance(Duration::from_secs(13 * 60));
    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Idle),
        None
    );
    clock.advance(Duration::from_secs(60));
    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Idle),
        Some(LockReason::Idle)
    );
}

#[rstest]
fn auto_lock_on_a_locked_session_decides_nothing(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, clock) = timed_session;
    session.lock(LockReason::User);
    let log = EventLog::new();
    session.subscribe(log.sink());

    clock.advance(Duration::from_secs(86_400));
    assert_eq!(
        session.auto_lock(&default_policy(), LockTrigger::Idle),
        None
    );
    assert!(log.events().is_empty());
}

// ── The hard tier (AC-3): password required after N days ─────────────────────

#[rstest]
#[case(Duration::from_secs(6 * 86_400), false)]
#[case(Duration::from_secs(7 * 86_400), true)]
#[case(Duration::from_secs(60 * 86_400), true)]
fn hard_lock_falls_at_the_configured_day_boundary(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
    #[case] elapsed: Duration,
    #[case] due: bool,
) {
    let (session, clock) = timed_session;
    clock.advance(elapsed);

    let reason = session.hard_lock_if_due(&default_policy());
    assert_eq!(reason, if due { Some(LockReason::Hard) } else { None });
    assert_eq!(
        session.status(),
        if due {
            VaultStatus::Locked
        } else {
            VaultStatus::Unlocked
        }
    );
}

#[rstest]
fn the_hard_lock_fires_the_same_event_every_lock_fires(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, clock) = timed_session;
    let log = EventLog::new();
    session.subscribe(log.sink());

    clock.advance(Duration::from_secs(8 * 86_400));
    session.hard_lock_if_due(&default_policy());

    assert_eq!(log.events(), vec![Event::DatabaseLocked]);
}

#[rstest]
fn unlocking_again_after_a_hard_lock_still_works(
    timed_session: (VaultSession, std::sync::Arc<ScriptedClock>),
) {
    let (session, clock) = timed_session;
    clock.advance(Duration::from_secs(8 * 86_400));
    session.hard_lock_if_due(&default_policy());

    // There is no quick-unlock tier yet (task-41); the password is the
    // only way back in, and it works.
    session
        .unlock(
            &corpus("test_db_kdbx4_with_password_argon2id.kdbx"),
            Some("demopass"),
            None,
        )
        .expect("a password unlock must work after a hard lock");
    assert_eq!(session.status(), VaultStatus::Unlocked);
}
