//! Local stdio transport with bounded input frames and protocol-only stdout.

use std::{
    fmt,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use rmcp::ServiceExt;
use starclock_agent_api::{
    error::{AgentError, AgentErrorCode},
    schema::SessionId,
    session::{
        AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry, OperationalClock,
        SessionIdSource,
    },
};
use tokio::io::{AsyncRead, ReadBuf};

use crate::server::StarclockMcp;

pub const MAX_STDIO_FRAME_BYTES: usize = 16 * 1024;

#[derive(Debug)]
pub enum StdioServeError {
    Runtime,
    Startup,
    Transport,
}

impl fmt::Display for StdioServeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime => formatter.write_str("the MCP async runtime could not start"),
            Self::Startup => formatter.write_str("the MCP application could not initialize"),
            Self::Transport => formatter.write_str("the MCP stdio transport stopped"),
        }
    }
}

impl std::error::Error for StdioServeError {}

pub fn serve() -> Result<(), StdioServeError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| StdioServeError::Runtime)?
        .block_on(serve_async())
}

async fn serve_async() -> Result<(), StdioServeError> {
    let factory = AgentSessionFactory::load_production().map_err(|_| StdioServeError::Startup)?;
    let registry = AgentSessionRegistry::new(
        factory.clone(),
        Arc::new(LocalClock::new()),
        Arc::new(LocalSessionIds::new()),
    );
    let owner = AgentSessionOwner::new("local-stdio", &format!("process-{}", std::process::id()))
        .map_err(|_| StdioServeError::Startup)?;
    let (stdin, stdout) = rmcp::transport::stdio();
    StarclockMcp::new(registry, factory, owner)
        .serve((BoundedReader::new(stdin), stdout))
        .await
        .map_err(|_| StdioServeError::Transport)?
        .waiting()
        .await
        .map(|_| ())
        .map_err(|_| StdioServeError::Transport)
}

struct LocalClock {
    started: Instant,
}

impl LocalClock {
    fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl OperationalClock for LocalClock {
    fn now_seconds(&self) -> u64 {
        self.started.elapsed().as_secs()
    }
}

struct LocalSessionIds {
    process: u32,
    started_nanos: u128,
    next: AtomicU64,
}

impl LocalSessionIds {
    fn new() -> Self {
        Self {
            process: std::process::id(),
            started_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            next: AtomicU64::new(1),
        }
    }
}

impl SessionIdSource for LocalSessionIds {
    fn next_session_id(&self) -> Result<SessionId, AgentError> {
        let ordinal = self.next.fetch_add(1, Ordering::Relaxed);
        SessionId::parse(&format!(
            "session_stdio_{:x}_{:x}_{ordinal:x}",
            self.process, self.started_nanos
        ))
        .map_err(|_| {
            AgentError::new(
                AgentErrorCode::AdapterFailure,
                "The local MCP session identity could not be allocated.",
                false,
                false,
            )
            .expect("static local identity error is bounded")
        })
    }
}

struct BoundedReader<R> {
    inner: R,
    current_frame_bytes: usize,
}

impl<R> BoundedReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            current_frame_bytes: 0,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for BoundedReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if buffer.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let mut storage = [0_u8; 8 * 1024];
        let capacity = buffer.remaining().min(storage.len());
        let mut staged = ReadBuf::new(&mut storage[..capacity]);
        match Pin::new(&mut self.inner).poll_read(context, &mut staged) {
            Poll::Ready(Ok(())) => {
                for byte in staged.filled() {
                    if *byte == b'\n' {
                        self.current_frame_bytes = 0;
                    } else {
                        self.current_frame_bytes = self.current_frame_bytes.saturating_add(1);
                        if self.current_frame_bytes > MAX_STDIO_FRAME_BYTES {
                            return Poll::Ready(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "MCP stdio frame exceeds the fixed limit",
                            )));
                        }
                    }
                }
                buffer.put_slice(staged.filled());
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tokio::io::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn bounded_reader_accepts_complete_frames_and_rejects_oversize_before_decode() {
        let mut valid = BoundedReader::new(Cursor::new(b"one\ntwo\n"));
        let mut output = Vec::new();
        valid.read_to_end(&mut output).await.unwrap();
        assert_eq!(output, b"one\ntwo\n");

        let mut oversized = vec![b'x'; MAX_STDIO_FRAME_BYTES + 1];
        oversized.push(b'\n');
        let mut bounded = BoundedReader::new(Cursor::new(oversized));
        assert!(bounded.read_to_end(&mut Vec::new()).await.is_err());
    }

    #[test]
    fn local_session_ids_are_distinct_valid_and_not_authority_credentials() {
        let source = LocalSessionIds::new();
        let first = source.next_session_id().unwrap();
        let second = source.next_session_id().unwrap();
        assert_ne!(first, second);
        assert!(first.as_str().starts_with("session_stdio_"));
    }
}
