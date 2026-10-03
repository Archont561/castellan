//! The IPC server suite: a real server on a real Unix socket in a scratch
//! dir, driven by clients that speak the same wire as the extension's
//! host. Every rule doc-2 §3 states as behavior — multiplexing, the
//! association handshake, silence on refusal, the panel's data, the kill
//! switch — is asserted here end to end.

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use castellan_ipc_server::{AssociationStore, IpcServer};
use castellan_protocol::{
    ConnectionState, Event, FaceKind, HostMessage, PROTOCOL_VERSION, RpcErrorCode,
};
use common::{TestClient, handler, hello_source, spawn_server, wait_until};
use rstest::rstest;

/// 32 bytes of key material, hex.
fn key_hex(byte: u8) -> String {
    (0..32).map(|_| format!("{byte:02x}")).collect()
}

#[test]
fn the_happy_path_enrolls_prompts_confirms_and_serves() {
    let (server, dir) = spawn_server("happy");

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));

    // The prompt appears in the panel before the client hears anything.
    wait_until("the enrollment prompt", || {
        !server.panel().pending.is_empty()
    });
    let pending = &server.panel().pending[0];
    assert_eq!(pending.key_id, "k-chrome");
    assert_eq!(pending.label, "Chrome on this machine");

    assert!(server.confirm("k-chrome"));
    let challenged = client.answer_challenge(&key_hex(1));
    assert_eq!(challenged, "k-chrome");
    client.expect_app_hello();

    // Ready: the panel shows the connection with its face and version.
    wait_until("a ready connection", || {
        server
            .panel()
            .connections
            .iter()
            .any(|conn| conn.state == ConnectionState::Ready)
    });
    let conn = &server.panel().connections[0];
    assert_eq!(conn.face, FaceKind::Chrome);
    assert_eq!(conn.client_version.as_deref(), Some("test-extension"));
    assert_eq!(conn.protocol_version, PROTOCOL_VERSION);
    assert_eq!(conn.last_request, None);

    // Requests flow, and the panel learns the last method name.
    let response = client.ping(7);
    assert_eq!(response.id, 7);
    assert!(response.result.is_some());
    wait_until("last_request to update", || {
        server.panel().connections[0].last_request.as_deref() == Some("ping")
    });

    // The remembered list is the panel's second tab.
    assert_eq!(server.panel().remembered.len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn two_browsers_multiplex_by_request_id() {
    let (server, dir) = spawn_server("multiplex");

    let mut chrome = TestClient::connect(&server);
    chrome.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    let mut firefox = TestClient::connect(&server);
    firefox.enroll(FaceKind::Firefox, "k-firefox", &key_hex(2));

    wait_until("both prompts", || server.panel().pending.len() == 2);
    assert!(server.confirm("k-chrome"));
    assert!(server.confirm("k-firefox"));
    chrome.answer_challenge(&key_hex(1));
    firefox.answer_challenge(&key_hex(2));
    chrome.expect_app_hello();
    firefox.expect_app_hello();

    // Interleaved calls, distinct ids: each stream answers its own.
    chrome.ping(100);
    firefox.ping(200);
    let chrome_answer = chrome.ping(101);
    let firefox_answer = firefox.ping(201);
    assert_eq!(chrome_answer.id, 101);
    assert_eq!(firefox_answer.id, 201);

    // Both live in the panel at once, one row per browser.
    wait_until("two ready connections", || {
        server.panel().connections.len() == 2
            && server
                .panel()
                .connections
                .iter()
                .all(|conn| conn.state == ConnectionState::Ready)
    });
    let faces: Vec<_> = server
        .panel()
        .connections
        .iter()
        .map(|conn| conn.face)
        .collect();
    assert!(faces.contains(&FaceKind::Chrome));
    assert!(faces.contains(&FaceKind::Firefox));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_client_that_claims_nothing_gets_silence() {
    let (server, dir) = spawn_server("unassociated");

    // A hello that never states who it is: no association claim at all.
    let mut client = TestClient::connect(&server);
    client.send(&castellan_protocol::ClientMessage::Hello {
        protocol_version: PROTOCOL_VERSION,
        face: Some(FaceKind::Chrome),
        client_version: Some("test-extension".to_string()),
        association: None,
    });
    client.expect_silence();
    // It never became a panel row: no association, no presence.
    assert!(server.panel().connections.is_empty());
    assert!(server.panel().pending.is_empty());

    // And the server is unbothered: the next connection is served.
    let mut honest = TestClient::connect(&server);
    honest.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn silence_is_the_answer_to_a_wrong_proof() {
    let (server, dir) = spawn_server("wrong-proof");

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-chrome"));

    match client.expect("challenge") {
        HostMessage::Challenge { key_id, nonce_hex } => {
            // Prove possession of the WRONG key.
            let proof = castellan_ipc_server::proof_hex(&key_hex(9), &nonce_hex);
            client.send(&castellan_protocol::ClientMessage::Proof {
                key_id,
                proof_hex: proof,
            });
        }
        other => panic!("expected a challenge, got {other:?}"),
    }
    client.expect_silence();

    // The refused connection leaves the panel; the server stays healthy.
    wait_until("the connection to leave the panel", || {
        server.panel().connections.is_empty()
    });
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_key_id_collision_gets_silence() {
    let (server, dir) = spawn_server("collision");

    // First contact for k1: enrolled and confirmed.
    let mut first = TestClient::connect(&server);
    first.enroll(FaceKind::Chrome, "k1", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k1"));
    first.answer_challenge(&key_hex(1));
    first.expect_app_hello();

    // A second browser claims the SAME id with DIFFERENT material.
    let mut impostor = TestClient::connect(&server);
    impostor.enroll(FaceKind::Firefox, "k1", &key_hex(2));
    impostor.expect_silence();

    // The panel never asked about the impostor.
    assert!(server.panel().pending.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn deny_closes_the_connection_in_silence() {
    let (server, dir) = spawn_server("deny");

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Edge, "k-edge", &key_hex(3));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.deny("k-edge"));

    client.expect_silence();
    wait_until("the prompt to leave", || server.panel().pending.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn an_unanswered_prompt_expires() {
    let (server, dir) = spawn_server("expiry");
    let server = server.with_timeouts(Duration::from_millis(300), Duration::from_secs(10));

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Brave, "k-brave", &key_hex(4));
    wait_until("the prompt", || !server.panel().pending.is_empty());

    let started = Instant::now();
    client.expect_silence();
    wait_until("the prompt to expire", || server.panel().pending.is_empty());
    // The prompt outlived the 300 ms timeout by no more than a few ticks.
    assert!(started.elapsed() < Duration::from_secs(2));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn an_unknown_key_is_invited_to_reenroll() {
    let (server, dir) = spawn_server("unknown-key");

    let mut client = TestClient::connect(&server);
    client.claim(FaceKind::Vivaldi, "k-vivaldi");
    match client.expect("unknown key") {
        HostMessage::UnknownKey { key_id } => assert_eq!(key_id, "k-vivaldi"),
        other => panic!("expected UnknownKey, got {other:?}"),
    }

    // Exactly one follow-up: a hello carrying a fresh enrollment.
    client.enroll(FaceKind::Vivaldi, "k-vivaldi", &key_hex(5));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-vivaldi"));
    client.answer_challenge(&key_hex(5));
    client.expect_app_hello();

    // And now the same connection serves requests.
    let response = client.ping(1);
    assert_eq!(response.id, 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_returning_client_claims_without_a_new_prompt() {
    let (server, dir) = spawn_server("returning");

    // First contact: enroll, confirm, answer, disconnect.
    let mut first = TestClient::connect(&server);
    first.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-chrome"));
    first.answer_challenge(&key_hex(1));
    first.expect_app_hello();
    drop(first);
    wait_until("the old connection to leave", || {
        server.panel().connections.is_empty()
    });

    // Returning: claim, challenge, proof — no prompt in between.
    let mut returning = TestClient::connect(&server);
    returning.claim(FaceKind::Chrome, "k-chrome");
    returning.answer_challenge(&key_hex(1));
    returning.expect_app_hello();
    assert!(server.panel().pending.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_kill_switch_closes_a_ready_connection_promptly() {
    let (server, dir) = spawn_server("kill");

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-chrome"));
    client.answer_challenge(&key_hex(1));
    client.expect_app_hello();

    wait_until("a ready connection", || {
        server
            .panel()
            .connections
            .iter()
            .any(|conn| conn.state == ConnectionState::Ready)
    });
    let id = server.panel().connections[0].id;

    let started = Instant::now();
    assert!(server.kill(id));
    client.expect_silence();
    // One read tick is 250 ms; four of them is the test's slack budget.
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "kill took {:?}, more than a few ticks",
        started.elapsed()
    );
    wait_until("the killed connection to leave the panel", || {
        server.panel().connections.is_empty()
    });
    // Killing a ghost is a false return, not an error.
    assert!(!server.kill(id));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_kill_during_approval_takes_the_prompt_down() {
    let (server, dir) = spawn_server("kill-pending");

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("a pending connection", || {
        !server.panel().pending.is_empty() && !server.panel().connections.is_empty()
    });
    let id = server.panel().connections[0].id;

    assert!(server.kill(id));
    client.expect_silence();
    // The connection AND its prompt are gone: no orphan prompts.
    wait_until("the prompt to leave with the connection", || {
        server.panel().pending.is_empty() && server.panel().connections.is_empty()
    });
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_shared_prompt_survives_one_kill() {
    let (server, dir) = spawn_server("shared-prompt");

    // Two connections, one key: one prompt, two waiters.
    let mut first = TestClient::connect(&server);
    first.enroll(FaceKind::Chrome, "k-shared", &key_hex(6));
    wait_until("the prompt", || server.panel().pending.len() == 1);
    let mut second = TestClient::connect(&server);
    second.enroll(FaceKind::Chrome, "k-shared", &key_hex(6));
    wait_until("both connections", || server.panel().connections.len() == 2);
    assert_eq!(server.panel().pending.len(), 1);

    // Kill the first; the prompt stays up for the second.
    let first_id = server.panel().connections[0].id;
    assert!(server.kill(first_id));
    first.expect_silence();
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        server.panel().pending.len(),
        1,
        "the shared prompt survives"
    );

    // The survivor confirms and completes the handshake.
    assert!(server.confirm("k-shared"));
    second.answer_challenge(&key_hex(6));
    second.expect_app_hello();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_stale_socket_is_rebound_a_live_one_is_not() {
    let dir = common::scratch("stale");
    let socket = dir.join("castellan.sock");
    let store = Arc::new(AssociationStore::in_memory());

    // Bind without starting, then drop: a stale socket file is left over
    // (a crashed app), and the next bind takes the path back.
    {
        let server =
            IpcServer::bind_with_store(&socket, Arc::clone(&store), handler(), hello_source())
                .expect("first bind");
        server.start();
        drop(server);
    }
    // The accept thread holds the listener, so the path is LIVE: a second
    // bind must refuse rather than steal it.
    let second = IpcServer::bind_with_store(&socket, store, handler(), hello_source());
    match second {
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse),
        Ok(_) => panic!("a live socket path was handed out twice"),
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn remembered_keys_survive_a_restart() {
    let dir = common::scratch("restart");
    let store_path = dir.join("associations.json");
    let first_socket = dir.join("one.sock");
    let second_socket = dir.join("two.sock");

    {
        let server = IpcServer::bind(&first_socket, store_path.clone(), handler(), hello_source())
            .expect("first server");
        server.start();
        let mut client = TestClient::connect(&server);
        client.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
        wait_until("the prompt", || !server.panel().pending.is_empty());
        assert!(server.confirm("k-chrome"));
        client.answer_challenge(&key_hex(1));
        client.expect_app_hello();
    }

    // A "restart": a fresh server, a fresh socket, the SAME store.
    let server = IpcServer::bind(&second_socket, store_path, handler(), hello_source())
        .expect("second server");
    server.start();
    let mut client = TestClient::connect(&server);
    client.claim(FaceKind::Chrome, "k-chrome");
    // No prompt this time: the key was remembered across the restart.
    assert!(server.panel().pending.is_empty());
    client.answer_challenge(&key_hex(1));
    client.expect_app_hello();
    assert_eq!(server.panel().remembered.len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn panel_changes_fire_a_signal() {
    let (server, dir) = spawn_server("changes");
    let fires = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&fires);
    server.subscribe_changes(Arc::new(move || {
        counter.fetch_add(1, Ordering::Relaxed);
    }));

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt AND its change signal", || {
        !server.panel().pending.is_empty() && fires.load(Ordering::Relaxed) >= 1
    });
    let before = fires.load(Ordering::Relaxed);
    assert!(server.confirm("k-chrome"));
    wait_until("the confirm to fire", || {
        fires.load(Ordering::Relaxed) > before
    });

    // Unsubscribe: silence is not a stream of no-ops.
    server.unsubscribe_changes(0);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn broadcasts_reach_ready_connections_only() {
    let (server, dir) = spawn_server("broadcast");

    // One ready connection.
    let mut ready = TestClient::connect(&server);
    ready.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-chrome"));
    ready.answer_challenge(&key_hex(1));
    ready.expect_app_hello();

    // One connection still waiting for approval.
    let mut waiting = TestClient::connect(&server);
    waiting.enroll(FaceKind::Firefox, "k-firefox", &key_hex(2));
    wait_until("the second prompt", || server.panel().pending.len() == 1);

    server.broadcast(Event::DatabaseLocked);
    match ready.expect("the broadcast") {
        HostMessage::Event {
            event: Event::DatabaseLocked,
        } => {}
        other => panic!("expected the lock event, got {other:?}"),
    }
    // The unapproved connection hears nothing: its first application
    // byte is the hello it is still earning.
    waiting.expect_nothing_for(Duration::from_millis(300));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_v2_hello_is_refused_in_silence_without_breaking_the_server() {
    let (server, dir) = spawn_server("v2-client");

    // The v2 shape: a hello with no face, no version, no association.
    let mut v2 = TestClient::connect(&server);
    v2.send(
        &serde_json::from_str::<castellan_protocol::ClientMessage>(
            r#"{"kind":"hello","protocol_version":2}"#,
        )
        .expect("v2 hellos still deserialize"),
    );
    v2.expect_silence();

    // The refusal is not a crash: the next connection is served.
    let mut v3 = TestClient::connect(&server);
    v3.enroll(FaceKind::Chrome, "k-chrome", &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-chrome"));
    v3.answer_challenge(&key_hex(1));
    v3.expect_app_hello();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_locked_vault_answer_flows_through_the_handler() {
    let (server, dir) = spawn_server("handler-error");

    let mut client = TestClient::connect(&server);
    client.enroll(FaceKind::Cli, "k-cli", &key_hex(8));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm("k-cli"));
    client.answer_challenge(&key_hex(8));
    client.expect_app_hello();

    // The test handler answers everything un-wired with VaultLocked, so
    // the error path of the channel is part of the suite.
    client.send(&castellan_protocol::ClientMessage::Request {
        request: castellan_protocol::RpcRequest {
            id: 42,
            method: castellan_protocol::RpcMethod::GetTotp {
                entry_id: "nope".to_string(),
            },
        },
    });
    match client.expect("the error response") {
        HostMessage::Response { response } => {
            assert_eq!(response.id, 42);
            let error = response.error.expect("the handler answers with an error");
            assert_eq!(error.code, RpcErrorCode::VaultLocked);
        }
        other => panic!("expected a response, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[rstest]
#[case::chrome(FaceKind::Chrome)]
#[case::edge(FaceKind::Edge)]
#[case::brave(FaceKind::Brave)]
#[case::vivaldi(FaceKind::Vivaldi)]
#[case::firefox(FaceKind::Firefox)]
#[case::safari(FaceKind::Safari)]
#[case::cli(FaceKind::Cli)]
fn every_face_completes_the_handshake(#[case] face: FaceKind) {
    let (server, dir) = spawn_server("faces");

    let mut client = TestClient::connect(&server);
    let key_id = format!("k-{face:?}");
    client.enroll(face, &key_id, &key_hex(1));
    wait_until("the prompt", || !server.panel().pending.is_empty());
    assert!(server.confirm(&key_id));
    client.answer_challenge(&key_hex(1));
    client.expect_app_hello();

    wait_until("the ready row", || {
        server
            .panel()
            .connections
            .iter()
            .any(|conn| conn.state == ConnectionState::Ready)
    });
    let conn = &server.panel().connections[0];
    assert_eq!(conn.face, face);
    let _ = std::fs::remove_dir_all(dir);
}
