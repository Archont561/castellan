//! The Windows stand-in for [`crate::server`] (compiled only on Windows).
//!
//! The real listener on Windows is a named pipe, and it lands with the
//! Windows CI lane that can run it — until then the shape of the API is
//! preserved so the desktop and mobile shells compile and can surface
//! "native channel unavailable" instead of guessing. Every constructor
//! fails with [`std::io::ErrorKind::Unsupported`]; no instance can ever
//! exist, so the remaining methods are unreachable no-ops that exist only
//! for name resolution in cross-platform callers.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use castellan_protocol::{Event, Hello, PanelSnapshot, RememberedKey, RpcRequest, RpcResponse};

/// Same shape as the Unix server's handler type.
pub type RequestHandler = Arc<dyn Fn(RpcRequest) -> RpcResponse + Send + Sync>;

/// Same shape as the Unix server's hello source.
pub type HelloSource = Arc<dyn Fn() -> Hello + Send + Sync>;

/// Same shape as the Unix server's change sink.
pub type ChangeSink = Arc<dyn Fn() + Send + Sync>;

/// The never-listening server. Uninstantiable: both constructors fail.
/// See the module docs.
pub struct IpcServer;

impl IpcServer {
    /// Always [`std::io::ErrorKind::Unsupported`] on Windows, for now.
    ///
    /// # Errors
    ///
    /// Always, with [`std::io::ErrorKind::Unsupported`].
    pub fn bind(
        _path: &Path,
        _store_path: PathBuf,
        _handler: RequestHandler,
        _hello: HelloSource,
    ) -> std::io::Result<Arc<Self>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "the Windows named-pipe listener lands with a Windows CI lane (task-9 notes)",
        ))
    }

    /// Always [`std::io::ErrorKind::Unsupported`] on Windows, for now.
    ///
    /// # Errors
    ///
    /// Always, with [`std::io::ErrorKind::Unsupported`].
    pub fn bind_with_store(
        _path: &Path,
        _store: Arc<crate::association::AssociationStore>,
        _handler: RequestHandler,
        _hello: HelloSource,
    ) -> std::io::Result<Arc<Self>> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "the Windows named-pipe listener lands with a Windows CI lane (task-9 notes)",
        ))
    }

    /// Never listens, so this is a no-op.
    #[must_use]
    pub fn with_timeouts(self: &Arc<Self>, _approval: Duration, _hello: Duration) -> Arc<Self> {
        Arc::clone(self)
    }

    /// Nothing is ever connected.
    #[must_use]
    pub fn panel(&self) -> PanelSnapshot {
        PanelSnapshot {
            connections: Vec::new(),
            pending: Vec::new(),
            remembered: Vec::new(),
            manifest_problems: Vec::new(),
        }
    }

    /// Nothing to surface on a platform with no listener.
    pub fn set_manifest_problems(&self, _problems: Vec<castellan_protocol::ManifestProblem>) {}

    /// No keys are remembered, because no store was ever loaded.
    #[must_use]
    pub fn remembered_keys(&self) -> Vec<RememberedKey> {
        Vec::new()
    }

    /// Nothing was ever pending.
    pub fn confirm(&self, _key_id: &str) -> bool {
        false
    }

    /// Nothing was ever pending.
    pub fn deny(&self, _key_id: &str) -> bool {
        false
    }

    /// Nothing is ever connected.
    pub fn kill(&self, _connection_id: u64) -> bool {
        false
    }

    /// Nothing to send to.
    pub fn broadcast(&self, _event: Event) {}

    /// No changes will ever arrive; the id is still valid to unsubscribe.
    pub fn subscribe_changes(&self, _sink: ChangeSink) -> u64 {
        0
    }

    /// Always works, vacuously.
    pub fn unsubscribe_changes(&self, _id: u64) {}

    /// The path the (never-started) server was asked for.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        PathBuf::new()
    }
}
