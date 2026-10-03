//! The association store: remembered keys, pending prompts, and the
//! HMAC-SHA256 proof that a returning connection holds the key it claims.
//!
//! One file on disk (`associations.json` beside wherever the shell puts it)
//! holds the remembered keys; the pending prompts are ephemeral on purpose —
//! a restart clears unanswered "wants to connect" prompts, which is the
//! safe direction (the extension re-enrolls and asks again).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use castellan_protocol::PendingKey;

/// HMAC-SHA256, the challenge answer's only moving part.
type HmacSha256 = Hmac<Sha256>;

/// A key's remembered state, as persisted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RememberedEntry {
    key_hex: String,
    label: String,
    added_at: u64,
}

/// The on-disk shape of the store.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct Persisted {
    remembered: BTreeMap<String, RememberedEntry>,
}

/// An enrollment waiting for the user.
#[derive(Debug)]
struct PendingEntry {
    key_hex: String,
    label: String,
    decision: Option<Decision>,
    /// Connections waiting on this prompt. The last one out removes the
    /// entry, so a killed connection cannot leave an orphan prompt — and
    /// a shared prompt (two connections, one key) outlives one kill.
    waiters: usize,
}

/// What the user decided about an enrollment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// The key is remembered; the connection may proceed.
    Confirmed,
    /// The key is refused; the connection closes.
    Denied,
}

/// How a wait for the user's decision ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitOutcome {
    /// The user confirmed; the key is remembered.
    Confirmed,
    /// The user denied; the connection closes.
    Denied,
    /// The prompt expired; the entry is removed and the connection closes.
    TimedOut,
    /// The connection was killed while waiting; the prompt stays up for a
    /// sibling connection (if any) and the killed connection closes.
    Cancelled,
}

/// What [`AssociationStore::enroll`] found, and therefore what the
/// connection should do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrollOutcome {
    /// The key is already remembered and the material matches: skip the
    /// prompt, go straight to the challenge.
    AlreadyRemembered,
    /// The key id is remembered but the material differs — an id collision
    /// or a replay attempt. The connection gets silence.
    KeyIdCollision,
    /// First contact: the prompt is up, wait for the user's decision.
    NewlyPending,
}

/// A remembered key, as the panel lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RememberedKey {
    /// The key's stable identifier (a fingerprint of its material).
    pub key_id: String,
    /// The label chosen at enrollment.
    pub label: String,
    /// When the user confirmed it, unix seconds.
    pub added_at: u64,
}

/// Errors the store can hit. Only persistence I/O; everything else is a
/// decision, not a failure.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The persistence file could not be read or written.
    #[error("association store I/O: {0}")]
    Io(#[from] std::io::Error),
    /// The persistence file exists but is not the expected JSON shape.
    #[error("association store is corrupt: {0}")]
    Corrupt(String),
}

/// The shared state under the store's mutex.
#[derive(Debug)]
struct Shared {
    remembered: BTreeMap<String, RememberedEntry>,
    pending: BTreeMap<String, PendingEntry>,
}

/// The store of association keys: remembered (persisted), pending (prompt
/// up, waiting), and the proof check that ties them together.
///
/// Shared between the connection threads (which enroll and wait) and the
/// shell's commands (which confirm and deny) through one mutex; the
/// condition variable is what lets a waiting connection hear the user's
/// decision without polling.
pub struct AssociationStore {
    path: Option<PathBuf>,
    state: Arc<(Mutex<Shared>, Condvar)>,
}

impl std::fmt::Debug for AssociationStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.0.lock().map_err(|_| std::fmt::Error)?;
        f.debug_struct("AssociationStore")
            .field("path", &self.path)
            .field("state", &*state)
            .finish()
    }
}

impl AssociationStore {
    /// A store that never touches disk — tests and in-memory shells.
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            path: None,
            state: Arc::new((
                Mutex::new(Shared {
                    remembered: BTreeMap::new(),
                    pending: BTreeMap::new(),
                }),
                Condvar::new(),
            )),
        }
    }

    /// A store persisted to `path`, loading whatever is already there.
    ///
    /// A missing file is an empty store, not an error — the first run has
    /// nothing remembered. A file that exists but does not parse is an
    /// error the caller must surface (silently re-enrolling everyone would
    /// hide corruption; silently keeping stale keys would hide a tamper).
    pub fn persisted(path: PathBuf) -> Result<Self, StoreError> {
        let remembered = match std::fs::read(&path) {
            Ok(bytes) => {
                let parsed: Persisted = serde_json::from_slice(&bytes)
                    .map_err(|error| StoreError::Corrupt(format!("{path:?}: {error}")))?;
                parsed.remembered
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            path: Some(path),
            state: Arc::new((
                Mutex::new(Shared {
                    remembered,
                    pending: BTreeMap::new(),
                }),
                Condvar::new(),
            )),
        })
    }

    /// The remembered keys, oldest first — the settings list.
    pub fn remembered(&self) -> Vec<RememberedKey> {
        let shared = self.state.0.lock().expect("association store");
        let mut keys: Vec<RememberedKey> = shared
            .remembered
            .iter()
            .map(|(key_id, entry)| RememberedKey {
                key_id: key_id.clone(),
                label: entry.label.clone(),
                added_at: entry.added_at,
            })
            .collect();
        keys.sort_by_key(|key| key.added_at);
        keys
    }

    /// The enrollments waiting for the user, oldest first — the prompts.
    pub fn pending(&self) -> Vec<PendingKey> {
        let shared = self.state.0.lock().expect("association store");
        let mut keys: Vec<PendingKey> = shared
            .pending
            .iter()
            .map(|(key_id, entry)| PendingKey {
                key_id: key_id.clone(),
                label: entry.label.clone(),
            })
            .collect();
        keys.sort_by(|a, b| a.key_id.cmp(&b.key_id));
        keys
    }

    /// Whether `key_id` is remembered — the claim path's first question.
    pub fn is_remembered(&self, key_id: &str) -> bool {
        self.state
            .0
            .lock()
            .expect("association store")
            .remembered
            .contains_key(key_id)
    }

    /// Enroll: first contact with a key. Idempotent for a key that is
    /// already pending with the same material (a browser that opened two
    /// connections while the prompt is up); an id collision with different
    /// material is silence.
    pub fn enroll(&self, key_id: &str, key_hex: &str, label: &str) -> EnrollOutcome {
        let mut shared = self.state.0.lock().expect("association store");
        if let Some(entry) = shared.remembered.get(key_id) {
            return if entry.key_hex.eq_ignore_ascii_case(key_hex) {
                EnrollOutcome::AlreadyRemembered
            } else {
                EnrollOutcome::KeyIdCollision
            };
        }
        if let Some(pending) = shared.pending.get_mut(key_id) {
            return if pending.key_hex.eq_ignore_ascii_case(key_hex) {
                // Already asked; both connections wait on the same prompt.
                // (A prompt the user just denied still carries its refusal —
                // the newcomer inherits it rather than re-asking instantly.)
                pending.waiters += 1;
                EnrollOutcome::NewlyPending
            } else {
                EnrollOutcome::KeyIdCollision
            };
        }
        shared.pending.insert(
            key_id.to_string(),
            PendingEntry {
                key_hex: key_hex.to_ascii_lowercase(),
                label: label.to_string(),
                decision: None,
                waiters: 1,
            },
        );
        EnrollOutcome::NewlyPending
    }

    /// Wait for the user's decision on a pending enrollment.
    ///
    /// The wait is sliced (250 ms) so a killed connection can leave the
    /// queue without a decision: pass its kill flag as `cancelled` and the
    /// wait returns [`WaitOutcome::Cancelled`] within one slice. A
    /// timeout removes the prompt — the extension re-enrolls and asks
    /// again — and a decision that raced ahead of the call is still
    /// honored. The last waiter out (denied or cancelled) removes the
    /// prompt, so it never outlives the connections it was asking for.
    pub fn wait_for_decision(
        &self,
        key_id: &str,
        timeout: Duration,
        cancelled: Option<&std::sync::atomic::AtomicBool>,
    ) -> WaitOutcome {
        use std::sync::atomic::Ordering;

        const SLICE: Duration = Duration::from_millis(250);
        let (mutex, signaled) = &*self.state;
        let mut shared = mutex.lock().expect("association store");
        let deadline = std::time::Instant::now() + timeout;
        loop {
            // The decision may have landed between enroll and this call.
            if shared.remembered.contains_key(key_id) {
                // confirm() already removed the entry; no waiter to release.
                return WaitOutcome::Confirmed;
            }
            match shared.pending.get(key_id) {
                Some(pending) => {
                    if let Some(decision) = pending.decision {
                        let outcome = match decision {
                            Decision::Confirmed => WaitOutcome::Confirmed,
                            Decision::Denied => WaitOutcome::Denied,
                        };
                        if decision == Decision::Denied {
                            // Refused: hand the refusal to this waiter, and
                            // the last one out takes the prompt down.
                            leave(&mut shared, key_id);
                        }
                        return outcome;
                    }
                }
                None => {
                    // A vanished entry that was never remembered: another
                    // waiter's timeout or exit cleaned it up. This waiter
                    // answers the same way — the prompt is gone either way.
                    return WaitOutcome::TimedOut;
                }
            }
            if let Some(flag) = cancelled {
                if flag.load(Ordering::Relaxed) {
                    // Killed: leave the wait. The prompt survives only if a
                    // sibling is still asking for it.
                    leave(&mut shared, key_id);
                    return WaitOutcome::Cancelled;
                }
            }
            let now = std::time::Instant::now();
            if now >= deadline {
                // The prompt expires: remove it so the panel stops asking,
                // and the connection closes (the client re-enrolls later).
                shared.pending.remove(key_id);
                return WaitOutcome::TimedOut;
            }
            let (guard, _left) = signaled
                .wait_timeout(shared, SLICE.min(deadline - now))
                .expect("association store");
            shared = guard;
        }
    }

    /// Confirm a pending enrollment: remember the key, wake the waiters.
    /// Returns whether there was a pending enrollment to confirm.
    pub fn confirm(&self, key_id: &str, now_unix: u64) -> bool {
        let mut shared = self.state.0.lock().expect("association store");
        let Some(pending) = shared.pending.remove(key_id) else {
            return false;
        };
        shared.remembered.insert(
            key_id.to_string(),
            RememberedEntry {
                key_hex: pending.key_hex,
                label: pending.label,
                added_at: now_unix,
            },
        );
        self.persist(&shared);
        drop(shared);
        self.state.1.notify_all();
        true
    }

    /// Deny a pending enrollment: wake the waiters with the refusal.
    /// Returns whether there was a pending enrollment to deny.
    ///
    /// The prompt itself comes down when its last waiter collects the
    /// refusal — the enrolling connection always follows its enroll with a
    /// wait in the same thread, so a denied prompt never outlives the
    /// connections it was asking for.
    pub fn deny(&self, key_id: &str) -> bool {
        let mut shared = self.state.0.lock().expect("association store");
        let Some(pending) = shared.pending.get_mut(key_id) else {
            return false;
        };
        pending.decision = Some(Decision::Denied);
        drop(shared);
        self.state.1.notify_all();
        true
    }

    /// Verify a challenge answer: `proof_hex` must be HMAC-SHA256 over
    /// `nonce_hex` keyed by the material remembered for `key_id`.
    ///
    /// Constant-time under the hood (`Mac::verify_slice`); unknown keys,
    /// malformed hex and wrong tags are all the same `false`.
    #[must_use]
    pub fn verify(&self, key_id: &str, nonce_hex: &str, proof_hex: &str) -> bool {
        let shared = self.state.0.lock().expect("association store");
        let Some(entry) = shared.remembered.get(key_id) else {
            return false;
        };
        verify_proof(&entry.key_hex, nonce_hex, proof_hex)
    }

    /// Write the remembered keys to disk, if the store is persisted.
    /// Best-effort by design: a failed write is logged by the caller's
    /// rules, but the in-memory state stays usable — losing persistence
    /// costs a re-enrollment, not a lockout.
    fn persist(&self, shared: &Shared) {
        let Some(path) = &self.path else {
            return;
        };
        let persisted = Persisted {
            remembered: shared.remembered.clone(),
        };
        if let Ok(bytes) = serde_json::to_vec_pretty(&persisted) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, bytes);
        }
    }
}

/// One waiter leaves a pending entry: decrement, and take the entry down
/// when the last one is gone. Callers hold the lock.
fn leave(shared: &mut std::sync::MutexGuard<'_, Shared>, key_id: &str) {
    let take_down = shared.pending.get_mut(key_id).is_some_and(|pending| {
        pending.waiters = pending.waiters.saturating_sub(1);
        pending.waiters == 0
    });
    if take_down {
        shared.pending.remove(key_id);
    }
}

/// Compute a challenge answer — the recipe any client follows, exposed so
/// the crate's own tests (and the future CLI) do not reimplement it.
#[must_use]
pub fn proof_hex(key_hex: &str, nonce_hex: &str) -> String {
    let key = hex::decode(key_hex).unwrap_or_default();
    let nonce = hex::decode(nonce_hex).unwrap_or_default();
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC accepts any key length");
    mac.update(&nonce);
    hex::encode(mac.finalize().into_bytes())
}

/// Verify a proof against key material, constant-time on the comparison.
#[must_use]
fn verify_proof(key_hex: &str, nonce_hex: &str, proof_hex: &str) -> bool {
    let Ok(key) = hex::decode(key_hex) else {
        return false;
    };
    let Ok(nonce) = hex::decode(nonce_hex) else {
        return false;
    };
    let Ok(proof) = hex::decode(proof_hex) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(&key) else {
        return false;
    };
    mac.update(&nonce);
    mac.verify_slice(&proof).is_ok()
}

/// A fresh 32-byte nonce, hex-encoded. Per connection, never stored.
///
/// # Errors
///
/// Only if the OS entropy source fails — at which point refusing new
/// connections is the only safe answer.
pub fn nonce_hex() -> std::io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(std::io::Error::other)?;
    Ok(hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn key_hex(byte: u8) -> String {
        hex::encode([byte; 32])
    }

    #[test]
    fn first_contact_is_newly_pending() {
        let store = AssociationStore::in_memory();
        assert_eq!(
            store.enroll("k1", &key_hex(1), "Chrome on this machine"),
            EnrollOutcome::NewlyPending
        );
        assert_eq!(
            store.pending(),
            vec![PendingKey {
                key_id: "k1".to_string(),
                label: "Chrome on this machine".to_string(),
            }]
        );
        assert!(store.remembered().is_empty());
    }

    #[test]
    fn same_material_while_pending_joins_the_prompt() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        // A second connection, same key: one prompt, two waiters.
        assert_eq!(
            store.enroll("k1", &key_hex(1), "Chrome"),
            EnrollOutcome::NewlyPending
        );
        assert_eq!(store.pending().len(), 1);
    }

    #[test]
    fn different_material_same_id_is_a_collision() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        // While pending:
        assert_eq!(
            store.enroll("k1", &key_hex(2), "Firefox"),
            EnrollOutcome::KeyIdCollision
        );
        // And against a remembered key:
        store.confirm("k1", 1000);
        assert_eq!(
            store.enroll("k1", &key_hex(2), "Firefox"),
            EnrollOutcome::KeyIdCollision
        );
        // Same material against a remembered key: no prompt, just proceed.
        assert_eq!(
            store.enroll("k1", &key_hex(1), "Chrome"),
            EnrollOutcome::AlreadyRemembered
        );
    }

    #[test]
    fn confirm_moves_the_key_to_remembered() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        assert!(store.confirm("k1", 1234));
        assert!(store.pending().is_empty());
        assert_eq!(
            store.remembered(),
            vec![RememberedKey {
                key_id: "k1".to_string(),
                label: "Chrome".to_string(),
                added_at: 1234,
            }]
        );
        // Confirming a second time finds nothing pending.
        assert!(!store.confirm("k1", 1235));
    }

    #[test]
    fn deny_wakes_the_waiter_with_a_refusal() {
        let store = Arc::new(AssociationStore::in_memory());
        store.enroll("k1", &key_hex(1), "Chrome");
        let waiter = Arc::clone(&store);
        let thread = std::thread::spawn(move || {
            waiter.wait_for_decision("k1", Duration::from_secs(30), None)
        });
        // The waiter must be inside the wait before the decision lands.
        std::thread::sleep(Duration::from_millis(100));
        assert!(store.deny("k1"));
        assert_eq!(thread.join().unwrap(), WaitOutcome::Denied);
        // The last waiter out took the prompt down.
        assert!(store.pending().is_empty());
    }

    #[test]
    fn a_decision_that_races_the_wait_is_honored() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        store.confirm("k1", 1);
        // The confirm landed before the wait even started.
        assert_eq!(
            store.wait_for_decision("k1", Duration::from_secs(1), None),
            WaitOutcome::Confirmed
        );
    }

    #[test]
    fn a_timeout_removes_the_prompt() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        let started = std::time::Instant::now();
        assert_eq!(
            store.wait_for_decision("k1", Duration::from_millis(200), None),
            WaitOutcome::TimedOut
        );
        assert!(started.elapsed() >= Duration::from_millis(200));
        assert!(store.pending().is_empty());
    }

    #[test]
    fn a_cancelled_wait_takes_the_prompt_down_when_it_is_the_last() {
        let store = Arc::new(AssociationStore::in_memory());
        store.enroll("k1", &key_hex(1), "Chrome");
        store.enroll("k1", &key_hex(1), "Chrome"); // two waiters
        let killed = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&killed);

        let waiter = Arc::clone(&store);
        let first = std::thread::spawn(move || {
            waiter.wait_for_decision("k1", Duration::from_secs(30), None)
        });
        let waiter = Arc::clone(&store);
        let second = std::thread::spawn(move || {
            waiter.wait_for_decision("k1", Duration::from_secs(30), Some(&flag))
        });

        std::thread::sleep(Duration::from_millis(100));
        killed.store(true, Ordering::Relaxed);
        // The cancelled waiter leaves; the prompt survives for its sibling.
        assert_eq!(second.join().unwrap(), WaitOutcome::Cancelled);
        assert_eq!(store.pending().len(), 1);
        // The sibling confirms; the prompt resolves normally.
        store.confirm("k1", 1);
        assert_eq!(first.join().unwrap(), WaitOutcome::Confirmed);
        assert!(store.pending().is_empty());
    }

    #[test]
    fn the_last_cancelled_waiter_removes_the_prompt() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        let killed = AtomicBool::new(true); // already dead when the wait starts
        assert_eq!(
            store.wait_for_decision("k1", Duration::from_secs(30), Some(&killed)),
            WaitOutcome::Cancelled
        );
        assert!(store.pending().is_empty());
    }

    #[test]
    fn a_deny_before_the_wait_starts_still_drains_when_the_waiter_arrives() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(1), "Chrome");
        // The decision lands in the window between enroll and wait — the
        // connection's own thread is that waiter, and it is always coming.
        assert!(store.deny("k1"));
        assert_eq!(
            store.wait_for_decision("k1", Duration::from_secs(1), None),
            WaitOutcome::Denied
        );
        // The waiter collected the refusal and took the prompt down.
        assert!(store.pending().is_empty());
    }

    #[test]
    fn verify_accepts_a_computed_proof_and_rejects_everything_else() {
        let store = AssociationStore::in_memory();
        store.enroll("k1", &key_hex(7), "Chrome");
        store.confirm("k1", 1);

        let nonce = nonce_hex().expect("entropy");
        let proof = proof_hex(&key_hex(7), &nonce);
        assert!(store.verify("k1", &nonce, &proof));
        // Wrong key material:
        assert!(!store.verify("k1", &nonce, &proof_hex(&key_hex(8), &nonce)));
        // Unknown key:
        assert!(!store.verify("k9", &nonce, &proof));
        // Tampered proof:
        let mut tampered = proof.clone();
        tampered.replace_range(0..1, if tampered.starts_with('0') { "1" } else { "0" });
        assert_ne!(tampered, proof);
        assert!(!store.verify("k1", &nonce, &tampered));
        // Malformed inputs:
        assert!(!store.verify("k1", "not-hex", &proof));
        assert!(!store.verify("k1", &nonce, "not-hex"));
    }

    #[test]
    fn nonces_never_repeat() {
        let a = nonce_hex().expect("entropy");
        let b = nonce_hex().expect("entropy");
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn a_persisted_store_round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("castellan-assoc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let path = dir.join("associations.json");

        let store = AssociationStore::persisted(path.clone()).expect("fresh store");
        store.enroll("k1", &key_hex(1), "Chrome");
        assert!(store.confirm("k1", 42));
        drop(store);

        // A restart loads the remembered key; the pending prompts are gone.
        let reopened = AssociationStore::persisted(path).expect("reopen");
        assert_eq!(
            reopened.remembered(),
            vec![RememberedKey {
                key_id: "k1".to_string(),
                label: "Chrome".to_string(),
                added_at: 42,
            }]
        );
        assert!(reopened.pending().is_empty());
        assert!(reopened.is_remembered("k1"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_missing_store_file_is_an_empty_store() {
        let dir =
            std::env::temp_dir().join(format!("castellan-assoc-missing-{}", std::process::id()));
        let store = AssociationStore::persisted(dir.join("associations.json"))
            .expect("missing file is not an error");
        assert!(store.remembered().is_empty());
    }

    #[test]
    fn a_corrupt_store_file_is_reported_not_swallowed() {
        let dir =
            std::env::temp_dir().join(format!("castellan-assoc-corrupt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let path = dir.join("associations.json");
        std::fs::write(&path, b"{not json").expect("write junk");
        match AssociationStore::persisted(path) {
            Err(StoreError::Corrupt(_)) => {}
            other => panic!("expected Corrupt, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    proptest! {
        #[test]
        fn proof_round_trip_proves_possession(key in "[0-9a-f]{64}", nonce in "[0-9a-f]{64}") {
            let store = AssociationStore::in_memory();
            store.enroll("k1", &key, "Chrome");
            store.confirm("k1", 1);
            let proof = proof_hex(&key, &nonce);
            prop_assert!(store.verify("k1", &nonce, &proof));
            // A proof computed for a different nonce proves nothing.
            let other_nonce = nonce_hex().expect("entropy");
            prop_assume!(other_nonce != nonce);
            prop_assert!(!store.verify("k1", &other_nonce, &proof));
        }
    }
}
