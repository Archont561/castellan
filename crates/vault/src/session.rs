//! The unlock/lock session: one state machine, shared by every face's
//! dispatcher (task-7). No Tauri types, no app concepts — the apps own
//! prompts and timers, this type owns the states and the rules.
//!
//! Time comes from a [`SessionClock`] so the tests script it: the session
//! reads the clock when it unlocks (anchoring the hard-lock deadline) and
//! when it evaluates policy, while the tests drive a fake clock through
//! the same paths the apps drive the real one. A session is `Send + Sync`
//! and safe to hold for the process lifetime.
//!
//! Locking is dropping: a locked session drops its [`VaultHandle`], and the
//! keepass values inside (including the retained key, needed for saving)
//! are `ZeroizeOnDrop` — there is no decrypted blob to chase. A *hard*
//! lock (reboot, app update, `hard_after` days) is the same drop with a
//! different reason: process death is inherently a hard lock, because a
//! new session always starts empty. The tiers differ in *policy*, and the
//! policy is what the apps configure.
//!
//! The event bus: subscribers hear the protocol's [`Event`] values
//! (`DatabaseUnlocked` / `DatabaseLocked`) the moment state changes, so
//! faces reflect lock state live. Subscribers are called after the state
//! transition completes, never while a lock is held, and never reentrantly.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use castellan_protocol::{EntrySummary, Event, VaultStatus};

use crate::{VaultError, VaultHandle, open};

/// A callback that hears session events as they happen.
pub type EventSink = std::sync::Arc<dyn Fn(&Event) + Send + Sync>;

/// Where the session reads the time. The default is the system clock;
/// the tests inject a scripted one so lock policy is decided by arithmetic,
/// never by how long the test happened to take.
pub trait SessionClock: Send + Sync {
    /// The current moment.
    fn now(&self) -> SystemTime;
}

/// The real clock: what every app session runs on.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SessionClock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// What the app is reporting when it asks whether to soft-lock.
///
/// The triggers are OS events the apps observe (window blur, system wake,
/// no user input); the session records when each landed and decides
/// against the [`LockPolicy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockTrigger {
    /// The window lost focus. Grace-windowed: alt-tabbing back and forth
    /// must not lock.
    Blur,
    /// The system woke from sleep. Usually immediate.
    Wake,
    /// A repeating tick with no user input since the last activity.
    Idle,
}

/// Why a session locked. Diagnostics and logs; the wire event stays
/// reason-free because faces only need the state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockReason {
    /// The user locked explicitly (button, `lock_database`, panic key).
    User,
    /// A new vault was unlocked over this one; the old session dropped.
    Replaced,
    /// The blur grace window elapsed.
    Blur,
    /// The system slept and the wake policy locked.
    Wake,
    /// No activity for the idle window.
    Idle,
    /// The hard-lock deadline elapsed: password required, no quick-unlock
    /// tier may apply (the hook task-41 builds on).
    Hard,
}

/// When a session should lock itself. Every knob is independently
/// optional; `None` disables that trigger entirely.
///
/// The defaults are the conservative shipping policy: a 5-minute blur
/// grace, lock on the first wake, 15 minutes of idle, and a hard lock
/// after 7 days. The tier UX (including the panic key and clipboard
/// hygiene) is task-15's to refine; those choices live here as data, so
/// refining them never touches the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockPolicy {
    /// Lock this long after the window blurs.
    pub after_blur: Option<Duration>,
    /// Lock this long after the system wakes.
    pub after_wake: Option<Duration>,
    /// Lock after this long without user activity.
    pub after_idle: Option<Duration>,
    /// Hard-lock this long after the vault was unlocked: past this point
    /// the master password is required again, whatever quick-unlock tier
    /// exists by then.
    pub hard_after: Option<Duration>,
}

impl Default for LockPolicy {
    fn default() -> Self {
        Self {
            after_blur: Some(Duration::from_secs(5 * 60)),
            after_wake: Some(Duration::ZERO),
            after_idle: Some(Duration::from_secs(15 * 60)),
            hard_after: Some(Duration::from_secs(7 * 86_400)),
        }
    }
}

/// What the session holds while a vault is open.
#[derive(Debug)]
struct Unlocked {
    handle: VaultHandle,
    unlocked_at: SystemTime,
    last_activity: SystemTime,
    /// When the current blur landed, if one is pending. `None` once the
    /// user is back (any activity cancels a pending grace).
    blurred_at: Option<SystemTime>,
    /// When the system last woke, if a wake is pending.
    woke_at: Option<SystemTime>,
}

/// The session's states. `Empty` and `Locked` answer identically to
/// entry requests (`vault_locked`) — the UI difference ("pick a file" vs
/// "enter your password") is what [`VaultSession::status`] is for.
#[derive(Debug, Default)]
enum State {
    /// Fresh session: no vault was ever opened on it.
    #[default]
    Empty,
    /// A vault is open; secrets are reachable through the handle. Boxed:
    /// the handle is the session's only heavyweight (a parsed database),
    /// and `State` sits in every lock call's stack frame.
    Unlocked(Box<Unlocked>),
    /// A vault was open and is no longer; the path is remembered so the
    /// prompt can offer the same file.
    Locked { last_path: PathBuf },
}

/// The in-memory vault session: the lock state machine every dispatcher
/// calls into. Clone-free by design — one session per process (per vault),
/// shared by handle.
///
/// ```
/// use castellan_vault::VaultSession;
/// use castellan_protocol::Event;
/// use std::sync::Arc;
///
/// let session = VaultSession::new();
/// let heard = Arc::new(std::sync::Mutex::new(Vec::new()));
/// let sink = {
///     let heard = Arc::clone(&heard);
///     Arc::new(move |event: &Event| heard.lock().unwrap().push(event.clone()))
/// };
/// session.subscribe(sink);
/// assert_eq!(session.status(), castellan_protocol::VaultStatus::NoDatabase);
/// ```
pub struct VaultSession {
    state: Mutex<State>,
    subscribers: Mutex<Vec<(u64, EventSink)>>,
    next_subscription: Mutex<u64>,
    clock: std::sync::Arc<dyn SessionClock>,
}

impl Default for VaultSession {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for VaultSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Subscribers are closures and cannot be Debug; the state is the
        // part a person debugging wants.
        f.debug_struct("VaultSession")
            .field("status", &self.status())
            .field("subscribers", &self.subscriber_count())
            .finish_non_exhaustive()
    }
}

impl VaultSession {
    /// A session on the system clock: the one every app constructs.
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(std::sync::Arc::new(SystemClock))
    }

    /// A session reading time from `clock` — the tests' scripted clock,
    /// and any future app that wants a deterministic timeline.
    #[must_use]
    pub fn with_clock(clock: std::sync::Arc<dyn SessionClock>) -> Self {
        Self {
            state: Mutex::new(State::Empty),
            subscribers: Mutex::new(Vec::new()),
            next_subscription: Mutex::new(0),
            clock,
        }
    }

    /// The vault status the protocol's handshake reports.
    #[must_use]
    pub fn status(&self) -> VaultStatus {
        match &*self.state.lock().expect("session state") {
            State::Empty => VaultStatus::NoDatabase,
            State::Unlocked(_) => VaultStatus::Unlocked,
            State::Locked { .. } => VaultStatus::Locked,
        }
    }

    /// The path of the vault this session last held, if any — the file the
    /// unlock prompt offers after a lock.
    #[must_use]
    pub fn last_path(&self) -> Option<PathBuf> {
        match &*self.state.lock().expect("session state") {
            State::Empty => None,
            State::Unlocked(unlocked) => Some(unlocked.handle.path().to_path_buf()),
            State::Locked { last_path } => Some(last_path.clone()),
        }
    }

    /// Unlock a vault into this session.
    ///
    /// A wrong key fails with [`VaultError::Credentials`] and changes
    /// nothing. Unlocking over an open session replaces it: the old
    /// handle drops (wiping its key material), the faces hear
    /// `DatabaseLocked` then `DatabaseUnlocked`, and the timing state
    /// starts over.
    pub fn unlock(
        &self,
        path: &Path,
        password: Option<&str>,
        keyfile: Option<&Path>,
    ) -> Result<VaultStatus, VaultError> {
        // Open before touching the session state: a failed attempt must
        // leave whatever is currently open exactly as it was.
        let handle = open(path, password, keyfile)?;
        let now = self.clock.now();

        let event = {
            let mut state = self.state.lock().expect("session state");
            let replaced = matches!(&*state, State::Unlocked(_));
            *state = State::Unlocked(Box::new(Unlocked {
                handle,
                unlocked_at: now,
                last_activity: now,
                blurred_at: None,
                woke_at: None,
            }));
            if replaced {
                Some(Event::DatabaseLocked)
            } else {
                None
            }
        };
        // Notify outside the state lock: a subscriber that calls back into
        // the session must not deadlock, and must see the new state.
        if let Some(event) = event {
            self.notify(&event);
        }
        self.notify(&Event::DatabaseUnlocked);
        Ok(self.status())
    }

    /// Lock now, whatever is open. Returns whether the session actually
    /// transitioned (locking an empty or locked session is a no-op that
    /// fires no events). The handle drops inside; its key material wipes.
    pub fn lock(&self, reason: LockReason) -> bool {
        let event = {
            let mut state = self.state.lock().expect("session state");
            match &*state {
                State::Unlocked(unlocked) => {
                    let last_path = unlocked.handle.path().to_path_buf();
                    *state = State::Locked { last_path };
                    Some(Event::DatabaseLocked)
                }
                State::Empty | State::Locked { .. } => None,
            }
        };
        // `reason` is diagnostics-only today; lock logging arrives with
        // task-15's tier UX.
        let _ = reason;
        if let Some(event) = event {
            self.notify(&event);
            true
        } else {
            false
        }
    }

    /// Every entry in the open vault, projected to the protocol's shape.
    ///
    /// Errors with [`VaultError::Locked`] (protocol code `vault_locked`)
    /// when there is no open vault — a locked vault answers list requests
    /// with a stable code, not a transport failure.
    pub fn entries(&self) -> Result<Vec<EntrySummary>, VaultError> {
        let state = self.state.lock().expect("session state");
        match &*state {
            State::Unlocked(unlocked) => Ok(unlocked.handle.entries()),
            State::Empty | State::Locked { .. } => Err(VaultError::Locked),
        }
    }

    /// Note user activity: resets the idle clock and cancels any pending
    /// blur or wake grace. The apps call this on every interaction their
    /// platform reports.
    pub fn touch(&self) {
        let now = self.clock.now();
        let mut state = self.state.lock().expect("session state");
        if let State::Unlocked(unlocked) = &mut *state {
            unlocked.last_activity = now;
            unlocked.blurred_at = None;
            unlocked.woke_at = None;
        }
    }

    /// Evaluate one soft-lock trigger against the policy, locking if it
    /// is due. Returns the reason if this call locked the session.
    ///
    /// `Blur` and `Wake` are grace-windowed: the first call records when
    /// the event landed (a grace of zero locks immediately), later calls
    /// measure against that moment — so the apps call this both on the OS
    /// event and on their repeating tick. `Idle` measures from the last
    /// [`VaultSession::touch`], or from unlock if there was none.
    pub fn auto_lock(&self, policy: &LockPolicy, trigger: LockTrigger) -> Option<LockReason> {
        let now = self.clock.now();
        let mut state = self.state.lock().expect("session state");
        let unlocked = match &mut *state {
            State::Unlocked(unlocked) => unlocked,
            State::Empty | State::Locked { .. } => return None,
        };
        let due = match trigger {
            LockTrigger::Blur => grace_elapsed(&mut unlocked.blurred_at, policy.after_blur, now),
            LockTrigger::Wake => grace_elapsed(&mut unlocked.woke_at, policy.after_wake, now),
            LockTrigger::Idle => policy
                .after_idle
                .is_some_and(|window| elapsed(unlocked.last_activity, now) >= window),
        };
        if !due {
            return None;
        }
        let last_path = unlocked.handle.path().to_path_buf();
        *state = State::Locked { last_path };
        drop(state);
        let reason = match trigger {
            LockTrigger::Blur => LockReason::Blur,
            LockTrigger::Wake => LockReason::Wake,
            LockTrigger::Idle => LockReason::Idle,
        };
        self.notify(&Event::DatabaseLocked);
        Some(reason)
    }

    /// Evaluate the hard-lock deadline, locking if it has elapsed.
    ///
    /// The deadline measures from the *password* unlock that opened this
    /// session. When a quick-unlock tier exists (task-41), it will need
    /// its own rule for which unlocks reset this clock; today every
    /// unlock is a password unlock, so the rule is simple.
    pub fn hard_lock_if_due(&self, policy: &LockPolicy) -> Option<LockReason> {
        let now = self.clock.now();
        let mut state = self.state.lock().expect("session state");
        let unlocked = match &mut *state {
            State::Unlocked(unlocked) => unlocked,
            State::Empty | State::Locked { .. } => return None,
        };
        let due = policy
            .hard_after
            .is_some_and(|window| elapsed(unlocked.unlocked_at, now) >= window);
        if !due {
            return None;
        }
        let last_path = unlocked.handle.path().to_path_buf();
        *state = State::Locked { last_path };
        drop(state);
        self.notify(&Event::DatabaseLocked);
        Some(LockReason::Hard)
    }

    /// Hear every unlock/lock state change. Returns the subscription id
    /// [`VaultSession::unsubscribe`] accepts.
    pub fn subscribe(&self, sink: EventSink) -> u64 {
        let mut next = self.next_subscription.lock().expect("subscription counter");
        *next += 1;
        let id = *next;
        drop(next);
        self.subscribers
            .lock()
            .expect("subscribers")
            .push((id, sink));
        id
    }

    /// How many subscribers are attached (diagnostics; the webview, the
    /// tray icon, the future IPC forwarder each hold one).
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.lock().expect("subscribers").len()
    }

    /// Stop a subscription; its sink hears nothing further.
    pub fn unsubscribe(&self, id: u64) {
        self.subscribers
            .lock()
            .expect("subscribers")
            .retain(|(their_id, _)| *their_id != id);
    }

    /// Deliver an event to every subscriber, in subscription order,
    /// holding no session locks while doing so.
    fn notify(&self, event: &Event) {
        let sinks: Vec<EventSink> = self
            .subscribers
            .lock()
            .expect("subscribers")
            .iter()
            .map(|(_, sink)| std::sync::Arc::clone(sink))
            .collect();
        for sink in sinks {
            sink(event);
        }
    }
}

/// Elapsed time that never panics on a clock set backwards: a negative
/// elapsed is zero, not an error — the decision "has the window elapsed?"
/// is the same either way.
fn elapsed(from: SystemTime, now: SystemTime) -> Duration {
    now.duration_since(from).unwrap_or(Duration::ZERO)
}

/// The blur/wake grace rule: the first report arms the clock (a zero
/// grace fires immediately), later reports measure against the armed
/// moment.
fn grace_elapsed(
    armed: &mut Option<SystemTime>,
    window: Option<Duration>,
    now: SystemTime,
) -> bool {
    let Some(window) = window else {
        return false;
    };
    let armed_at = *armed.get_or_insert(now);
    elapsed(armed_at, now) >= window
}
