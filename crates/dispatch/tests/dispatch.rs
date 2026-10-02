//! Public dispatcher behavior at the transport-independent boundary.

use castellan_dispatch::dispatch;
use castellan_protocol::{RpcErrorCode, RpcMethod, RpcRequest, RpcResult};

#[test]
fn preserves_the_request_id_and_correlated_result() {
    let response = dispatch(RpcRequest {
        id: 42,
        method: RpcMethod::Ping {},
    });

    assert_eq!(response.id, 42);
    assert!(matches!(response.result, Some(RpcResult::Ping {})));
    assert!(response.error.is_none());
}

#[test]
fn generates_a_passphrase_once_for_every_native_face() {
    let response = dispatch(RpcRequest {
        id: 7,
        method: RpcMethod::GeneratePassphrase {
            words: 4,
            separator: "-".into(),
        },
    });

    let Some(RpcResult::GeneratePassphrase { value }) = response.result else {
        panic!("generate_passphrase returned the wrong result variant");
    };
    assert_eq!(value.split('-').count(), 4);
    assert!(response.error.is_none());
}

#[test]
fn returns_application_errors_inside_the_response_envelope() {
    let response = dispatch(RpcRequest {
        id: 9,
        method: RpcMethod::GetTotp {
            entry_id: "entry-id".into(),
        },
    });

    assert_eq!(response.id, 9);
    assert!(response.result.is_none());
    assert_eq!(
        response.error.expect("missing error").code,
        RpcErrorCode::NotImplemented
    );
}
