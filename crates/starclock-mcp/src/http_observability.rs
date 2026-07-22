//! Non-authoritative HTTP lifecycle and fixed low-cardinality metrics.

use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU64, Ordering},
};

use axum::{
    body::Body,
    http::{Response, StatusCode, header::CONTENT_TYPE},
};
use serde::Serialize;
use tokio::sync::Notify;

pub const HEALTH_PATH: &str = "/healthz";
pub const READINESS_PATH: &str = "/readyz";
pub const METRICS_PATH: &str = "/metrics";
pub const DRAIN_TIMEOUT_SECONDS: u64 = 10;

const RUNNING: u8 = 0;
const DRAINING: u8 = 1;
const STOPPED: u8 = 2;

#[derive(Clone)]
pub(crate) struct HttpOperations {
    inner: Arc<OperationsInner>,
}

struct OperationsInner {
    lifecycle: AtomicU8,
    in_flight: AtomicU64,
    requests_started: AtomicU64,
    requests_completed: AtomicU64,
    drain_rejections: AtomicU64,
    worker_rejections: AtomicU64,
    rate_rejections: AtomicU64,
    drained: Notify,
}

pub(crate) struct RequestGuard {
    operations: HttpOperations,
}

#[derive(Serialize)]
struct MetricsSnapshot {
    schema_revision: &'static str,
    authoritative: bool,
    ready: bool,
    draining: bool,
    in_flight: u64,
    requests_started: u64,
    requests_completed: u64,
    drain_rejections: u64,
    worker_rejections: u64,
    rate_rejections: u64,
}

impl HttpOperations {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(OperationsInner {
                lifecycle: AtomicU8::new(RUNNING),
                in_flight: AtomicU64::new(0),
                requests_started: AtomicU64::new(0),
                requests_completed: AtomicU64::new(0),
                drain_rejections: AtomicU64::new(0),
                worker_rejections: AtomicU64::new(0),
                rate_rejections: AtomicU64::new(0),
                drained: Notify::new(),
            }),
        }
    }

    pub(crate) fn start_request(&self) -> Option<RequestGuard> {
        if self.inner.lifecycle.load(Ordering::Acquire) != RUNNING {
            increment(&self.inner.drain_rejections);
            return None;
        }
        self.inner.in_flight.fetch_add(1, Ordering::AcqRel);
        if self.inner.lifecycle.load(Ordering::Acquire) != RUNNING {
            self.finish_request(false);
            increment(&self.inner.drain_rejections);
            return None;
        }
        increment(&self.inner.requests_started);
        Some(RequestGuard {
            operations: self.clone(),
        })
    }

    pub(crate) fn begin_draining(&self) {
        let _ = self.inner.lifecycle.compare_exchange(
            RUNNING,
            DRAINING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        if self.inner.in_flight.load(Ordering::Acquire) == 0 {
            self.inner.drained.notify_waiters();
        }
    }

    pub(crate) fn stop(&self) {
        self.inner.lifecycle.store(STOPPED, Ordering::Release);
        self.inner.drained.notify_waiters();
    }

    #[cfg(test)]
    pub(crate) async fn wait_until_drained(&self) {
        loop {
            let notified = self.inner.drained.notified();
            if self.inner.in_flight.load(Ordering::Acquire) == 0 {
                return;
            }
            notified.await;
        }
    }

    pub(crate) fn record_worker_rejection(&self) {
        increment(&self.inner.worker_rejections);
    }

    pub(crate) fn record_rate_rejection(&self) {
        increment(&self.inner.rate_rejections);
    }

    pub(crate) fn health_response(&self) -> Response<Body> {
        json_response(
            StatusCode::OK,
            serde_json::json!({"status":"live"}).to_string(),
        )
    }

    pub(crate) fn readiness_response(&self) -> Response<Body> {
        let ready = self.inner.lifecycle.load(Ordering::Acquire) == RUNNING;
        json_response(
            if ready {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            },
            serde_json::json!({"status": if ready { "ready" } else { "draining" }}).to_string(),
        )
    }

    pub(crate) fn metrics_response(&self) -> Response<Body> {
        let lifecycle = self.inner.lifecycle.load(Ordering::Acquire);
        let snapshot = MetricsSnapshot {
            schema_revision: "starclock.mcp-http-metrics.v1",
            authoritative: false,
            ready: lifecycle == RUNNING,
            draining: lifecycle == DRAINING,
            in_flight: self.inner.in_flight.load(Ordering::Acquire),
            requests_started: self.inner.requests_started.load(Ordering::Relaxed),
            requests_completed: self.inner.requests_completed.load(Ordering::Relaxed),
            drain_rejections: self.inner.drain_rejections.load(Ordering::Relaxed),
            worker_rejections: self.inner.worker_rejections.load(Ordering::Relaxed),
            rate_rejections: self.inner.rate_rejections.load(Ordering::Relaxed),
        };
        let body = serde_json::to_string(&snapshot)
            .unwrap_or_else(|_| "{\"schema_revision\":\"starclock.mcp-http-metrics.v1\"}".into());
        json_response(StatusCode::OK, body)
    }

    fn finish_request(&self, completed: bool) {
        if completed {
            increment(&self.inner.requests_completed);
        }
        if self.inner.in_flight.fetch_sub(1, Ordering::AcqRel) == 1
            && self.inner.lifecycle.load(Ordering::Acquire) != RUNNING
        {
            self.inner.drained.notify_waiters();
        }
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        self.operations.finish_request(true);
    }
}

fn increment(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

fn json_response(status: StatusCode, body: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .expect("bounded operational response is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn draining_rejects_new_work_and_waits_for_admitted_work_only() {
        let operations = HttpOperations::new();
        let request = operations.start_request().unwrap();
        operations.begin_draining();
        assert!(operations.start_request().is_none());
        assert_eq!(operations.readiness_response().status(), 503);
        let waiting = operations.wait_until_drained();
        tokio::pin!(waiting);
        tokio::select! {
            () = &mut waiting => panic!("admitted request drained before its guard was dropped"),
            () = tokio::task::yield_now() => {}
        }
        drop(request);
        operations.wait_until_drained().await;
        let metrics = operations.metrics_response();
        assert_eq!(metrics.status(), 200);
        operations.stop();
    }
}
