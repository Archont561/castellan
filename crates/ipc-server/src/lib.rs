//! The app-side half of the native channel (doc-2): the socket server
//! that multiplexes browser connections, the association handshake in
//! front of it, and the data the connected-browsers panel renders.
//!
//! A library crate, not an app-shell concern, so every rule here is
//! testable without a webview — the desktop and mobile shells construct
//! [`IpcServer`], hand it a request handler and their session's event
//! stream, and surface its [`PanelSnapshot`] in their UI.
//!
//! # The handshake in one breath
//!
//! 1. The extension's native host connects to the well-known socket and
//!    sends a hello carrying its *face* (which browser), its version, and
//!    an association claim.
//! 2. First contact claims [`AssociationClaim::Enroll`]: key material over
//!    the local socket, once. The app checks the peer's credentials
//!    (same user — see [`same_user`]) *before* reading anything, then
//!    prompts: the enrollment sits in the panel until the user confirms,
//!    the connection denies, or the prompt expires.
//! 3. Returning clients claim [`AssociationClaim::Claim`] with their key
//!    id. The app answers with a nonce challenge; the client replies with
//!    an HMAC-SHA256 over the nonce keyed by the enrolled material. Only
//!    then does the app send its own hello and start serving requests.
//!
//! # Why an HMAC proof and not a signature
//!
//! doc-2 sketches an Ed25519 proof, but no signature crate is vendored
//! and hand-rolling one is off the table. A symmetric HMAC-SHA256
//! possession proof gives the same property that matters here — the
//! returning client proves it is the one that enrolled, without
//! re-sending secret material — because the channel is a same-user local
//! socket. The wire shapes are deliberately scheme-agnostic (enroll =
//! key material, claim = bare id), so swapping in a signature later
//! rewrites [`association::verify`] and [`association::proof_hex`] and
//! nothing else.
//!
//! # The silence rule
//!
//! Every refusal — a client that never says hello, an unknown key that
//! does not re-enroll, a wrong proof, a key-id collision, a denied
//! prompt — closes the connection with *zero application bytes*. An
//! unassociated host learns nothing about the app, not even why it was
//! refused (doc-2 §3). The one exception is [`HostMessage::UnknownKey`],
//! which invites exactly one follow-up hello carrying a fresh enrollment.
//!
//! # Same-user enforcement
//!
//! The socket directory is created 0700 per-user, and on accept the
//! server checks the kernel's word for the peer's identity against its
//! own: `SO_PEERCRED` on Linux, `getpeereid` on macOS/iOS. The decision
//! is [`same_user`], pure, because no test can spawn a second uid in CI.
//!
//! # The panel is data
//!
//! The server owns no UI. It exposes [`IpcServer::panel`] (a
//! [`PanelSnapshot`]: connections, pending prompts, remembered keys) and
//! a change-signal subscription; the desktop shell renders that, and the
//! panel's buttons call back into [`IpcServer::confirm`],
//! [`IpcServer::deny`], and [`IpcServer::kill`].

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod association;
#[cfg(unix)]
mod server;
#[cfg(windows)]
mod server_stub;

#[cfg(unix)]
pub use server::{ChangeSink, HelloSource, IpcServer, RequestHandler};
#[cfg(windows)]
pub use server_stub::{ChangeSink, HelloSource, IpcServer, RequestHandler};

pub use association::{
    AssociationStore, Decision, EnrollOutcome, RememberedKey, StoreError, WaitOutcome, nonce_hex,
    proof_hex,
};
pub use castellan_protocol::{ConnectionInfo, ConnectionState, PanelSnapshot};

/// The same-user decision, pure so the rule is testable without a second
/// uid: does a peer uid match the server's? Cross-user connections are
/// refused whatever else they say (doc-2 §3).
#[must_use]
pub fn same_user(peer_uid: u32, server_uid: u32) -> bool {
    peer_uid == server_uid
}

#[cfg(test)]
mod tests {
    use super::same_user;
    use rstest::rstest;

    #[rstest]
    #[case::identical(1000, 1000, true)]
    #[case::off_by_one(1000, 1001, false)]
    #[case::root_vs_user(0, 1000, false)]
    #[case::both_root(0, 0, true)]
    fn same_user_decides_by_equality(
        #[case] peer: u32,
        #[case] server: u32,
        #[case] expected: bool,
    ) {
        assert_eq!(same_user(peer, server), expected);
    }
}
