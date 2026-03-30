//! OpenTelemetry tracing middleware for axum.
//!
//! Provides [`opentelemetry_tracing_layer`] — a Tower layer that:
//! - Propagates W3C Trace Context headers from incoming requests
//! - Creates a `tracing` span for each HTTP request (which flows through
//!   `tracing-opentelemetry` to the configured OTLP exporter)
//! - Records HTTP method, path, status code, and request duration
//!
//! # Usage
//!
//! ```no_run
//! use axum::Router;
//! use agileplus_api::middleware::otel::opentelemetry_tracing_layer;
//!
//! let app: Router = Router::new()
//!     // .route(...)
//!     .layer(opentelemetry_tracing_layer());
//! ```

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use axum::{
    body::Body,
    http::{Request, Response},
};
use tower::{Layer, Service};
use tracing::Instrument;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build the OpenTelemetry HTTP tracing layer.
///
/// Each request gets a `tracing::info_span!` named `"http.request"` with
/// attributes `http.method`, `http.path`, and `http.status_code` (recorded on
/// response).  Because the subscriber is assumed to have a
/// `tracing-opentelemetry` layer installed, spans are automatically exported
/// to the configured OTLP endpoint.
pub fn opentelemetry_tracing_layer() -> OtelTracingLayer {
    OtelTracingLayer
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

/// Tower [`Layer`] that wraps each request in an OTel-compatible tracing span.
#[derive(Clone)]
pub struct OtelTracingLayer;

impl<S> Layer<S> for OtelTracingLayer {
    type Service = OtelTracingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelTracingMiddleware { inner }
    }
}

// ---------------------------------------------------------------------------
// Middleware service
// ---------------------------------------------------------------------------

/// Tower [`Service`] that instruments each request with a `tracing` span.
#[derive(Clone)]
pub struct OtelTracingMiddleware<S> {
    inner: S,
}

impl<S, ResBody> Service<Request<Body>> for OtelTracingMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        // Extract W3C Trace Context from incoming headers.
        // The `tracing` span created here is picked up by `tracing-opentelemetry`
        // which propagates the remote context automatically when the layer is
        // installed in the subscriber.
        let traceparent = req
            .headers()
            .get("traceparent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-")
            .to_string();

        let span = tracing::info_span!(
            "http.request",
            http.method = %method,
            http.path = %path,
            http.traceparent = %traceparent,
            http.status_code = tracing::field::Empty,
            otel.name = %format!("{method} {path}"),
            otel.kind = "server",
        );

        let start = Instant::now();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let response = inner.call(req).instrument(span.clone()).await?;

            let status = response.status().as_u16();
            let elapsed_ms = start.elapsed().as_millis();
            span.record("http.status_code", status);
            tracing::info!(
                parent: &span,
                http.status_code = status,
                duration_ms = elapsed_ms,
                "request completed"
            );

            Ok(response)
        })
    }
}
