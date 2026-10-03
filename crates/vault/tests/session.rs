//! The unlock/lock session: the one state machine every face's dispatcher
//! calls into (task-7). Time is data here — every policy decision takes its
//! `now` as a parameter, so these tests script time as literal values
//! instead of sleeping, and the apps pass `SystemTime::now()` in production.
//!
//! The corpus cases run against the generated KDBX 4.1 fixtures
//! (`common/mod.rs`: one factory, a case table, exact expectations) plus
//! the two committed anchors in `fixtures/` — the KeePassXC-authored 4.1
//! file and the KDBX 3.1 read-compatibility file, which no generator can
//! author.

mod common;

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

/// A session unlocked against the generated Argon2id case, ready to be
/// poked — the pytest-fixture pattern, via rstest. Fresh per test: a test
/// that locks its session cannot leak into the next one. The vault's path
/// comes along because the lock tests assert `last_path` against it.
#[fixture]
fn unlocked_session() -> (VaultSession, std::path::PathBuf) {
    let fixture = common::build(common::case("argon2id-aes-gzip"), "session-fixture");
    let session = VaultSession::new();
    session
        .unlock(&fixture.vault, fixture.password, None)
        .expect("the generated fixture must unlock");
    (session, fixture.vault)
}

// ── The corpus: real files, real keys (AC-1) ─────────────────────────────────

/// What a committed anchor fixture is and what the projection should find
/// inside — read off the files themselves, because no generator wrote
/// them (that is their whole point).
struct AnchorCase {
    file: &'static str,
    password: &'static str,
    expected_titles: &'static [&'static str],
}

const ANCHORS: &[AnchorCase] = &[
    AnchorCase {
        file: "test_db_kdbx41_features.kdbx",
        password: "demopass",
        // Authored by KeePassXC 2.7.12 — the external-authority anchor.
        expected_titles: &["tagged-entry-41", "ayyyyo"],
    },
    AnchorCase {
        file: "test_db_with_password.kdbx",
        password: "demopass",
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
];

#[rstest]
#[case(&common::GENERATED_CORPUS[0])]
#[case(&common::GENERATED_CORPUS[1])]
#[case(&common::GENERATED_CORPUS[2])]
#[case(&common::GENERATED_CORPUS[3])]
#[case(&common::GENERATED_CORPUS[4])]
#[case(&common::GENERATED_CORPUS[5])]
#[case(&common::GENERATED_CORPUS[6])]
#[case(&common::GENERATED_CORPUS[7])]
#[case(&common::GENERATED_CORPUS[8])]
#[case(&common::GENERATED_CORPUS[9])]
#[case(&common::GENERATED_CORPUS[10])]
#[case(&common::GENERATED_CORPUS[11])]
fn unlocks_every_generated_case_and_projects_its_entries(#[case] case: &common::KdbxCase) {
    let fixture = common::build(case, "unlock-matrix");
    let session = VaultSession::new();

    session
        .unlock(&fixture.vault, fixture.password, fixture.keyfile.as_deref())
        .expect("a generated case must unlock with its specified key");

    assert_eq!(session.status(), VaultStatus::Unlocked);
    let entries = session
        .entries()
        .expect("an unlocked session lists entries");
    let mut titles: Vec<&str> = entries.iter().map(|entry| entry.title.as_str()).collect();
    titles.sort_unstable();
    assert_eq!(
        titles,
        case.expected_titles(),
        "projected titles for {}",
        case.name
    );
}

#[rstest]
#[case(&ANCHORS[0])]
#[case(&ANCHORS[1])]
fn unlocks_the_committed_anchors_and_projects_their_entries(#[case] case: &AnchorCase) {
    let session = VaultSession::new();

    session
        .unlock(&common::anchor(case.file), Some(case.password), None)
        .expect("an anchor fixture must unlock with its documented key");

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
fn a_wrong_password_fails_with_the_stable_bad_credentials_code() {
    let fixture = common::build(common::case("argon2id-aes-gzip"), "wrong-password");
    let session = VaultSession::new();

    let error = session
        .unlock(&fixture.vault, Some("not the password"), None)
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
        .unlock(&fixture.vault, fixture.password, None)
        .expect("the correct key must unlock after a failed attempt");
}

// KDBX 3 fails at decryption rather than the key check — wrong key all
// the same, and the user's question ("is it the password?") is identical.
#[rstest]
fn a_wrong_password_on_kdbx3_fails_the_same_stable_way() {
    let path = common::anchor("test_db_with_password.kdbx");
    let session = VaultSession::new();

    let error = session
        .unlock(&path, Some("also not the password"), None)
        .expect_err("a wrong key must not unlock a KDBX 3 vault");

    assert!(
        matches!(error, VaultError::Credentials),
        "wrong KDBX 3 key must map to Credentials, got {error:?}"
    );
    assert_eq!(error.error_code(), Some(RpcErrorCode::BadCredentials));
    session
        .unlock(&path, Some("demopass"), None)
        .expect("the correct key must unlock after a failed attempt");
}

#[rstest]
fn a_password_is_not_the_keyfiles_key() {
    let fixture = common::build(common::case("keyfile-only-raw32"), "keyfile-only");
    let session = VaultSession::new();

    let error = session
        .unlock(&fixture.vault, Some("demopass"), None)
        .expect_err("a password cannot open a keyfile-only vault");
    assert_eq!(error.error_code(), Some(RpcErrorCode::BadCredentials));

    session
        .unlock(&fixture.vault, None, fixture.keyfile.as_deref())
        .expect("the keyfile must unlock it");
    assert_eq!(session.status(), VaultStatus::Unlocked);
}

// ── Locking and the wipe (AC-2) ──────────────────────────────────────────────

#[rstest]
fn lock_wipes_the_session_and_answers_with_vault_locked(
    unlocked_session: (VaultSession, std::path::PathBuf),
) {
    let (session, vault) = unlocked_session;
    assert!(session.lock(LockReason::User));

    assert_eq!(session.status(), VaultStatus::Locked);
    let error = session
        .entries()
        .expect_err("a locked session has no entries to list");
    assert_eq!(error.error_code(), Some(RpcErrorCode::VaultLocked));
    // The UI remembers which file to offer: locked, not amnesiac.
    assert_eq!(session.last_path(), Some(vault));
}

#[rstest]
fn locking_an_already_locked_session_is_a_no_op(
    unlocked_session: (VaultSession, std::path::PathBuf),
) {
    let (session, _) = unlocked_session;
    session.lock(LockReason::User);

    let log = EventLog::new();
    session.subscribe(log.sink());
    assert!(!session.lock(LockReason::User));

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

    let fixture = common::build(common::case("argon2id-aes-gzip"), "inline-unlock");
    session
        .unlock(&fixture.vault, fixture.password, None)
        .expect("the generated fixture must unlock");
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

    let fixture = common::build(common::case("argon2id-aes-gzip"), "failed-unlock");
    let _ = session.unlock(&fixture.vault, Some("wrong"), None);

    assert!(log.events().is_empty());
}

#[rstest]
fn subscribers_can_leave_and_stop_hearing(unlocked_session: (VaultSession, std::path::PathBuf)) {
    let (unlocked_session, _) = unlocked_session;
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
fn unlocking_over_an_open_session_replaces_it_and_both_events_fire(
    unlocked_session: (VaultSession, std::path::PathBuf),
) {
    let (session, _) = unlocked_session;
    let replacement = common::build(common::case("aeskdf-aes"), "replacement-vault");
    let log = EventLog::new();
    session.subscribe(log.sink());

    session
        .unlock(&replacement.vault, replacement.password, None)
        .expect("the second vault must unlock");

    // The old vault locks (its key material drops), then the new one opens.
    assert_eq!(
        log.events(),
        vec![Event::DatabaseLocked, Event::DatabaseUnlocked]
    );
    assert_eq!(session.last_path(), Some(replacement.vault));
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
    let fixture = common::build(common::case("argon2id-aes-gzip"), "inline-unlock");
    session
        .unlock(&fixture.vault, fixture.password, None)
        .expect("the generated fixture must unlock");
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
    let fixture = common::build(common::case("argon2id-aes-gzip"), "after-hard-lock");
    session
        .unlock(&fixture.vault, fixture.password, None)
        .expect("a password unlock must work after a hard lock");
    assert_eq!(session.status(), VaultStatus::Unlocked);
}
