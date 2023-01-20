use crate::{application::Application, trace::TraceContext, BoxError, Map, Uuid};
use reqwest::{
    header::{self, HeaderMap, HeaderName},
    Client, IntoUrl, Method, Request, Response, Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::{ReqwestOtelSpanBackend, TracingMiddleware};
use serde_json::Value;
use std::{
    borrow::Cow,
    net::IpAddr,
    str::FromStr,
    sync::OnceLock,
    time::{Duration, Instant},
};
use task_local_extensions::Extensions;
use tracing::{field::Empty, Span};

pub(super) fn init<APP: Application + ?Sized>() {
    let name = APP::name();
    let version = APP::version();
    let mut client_builder = Client::builder()
        .user_agent(format!("ZinoBot/1.0 {name}/{version}"))
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true);
    if let Some(http_client) = APP::config().get("http-client").and_then(|v| v.as_table()) {
        if let Some(timeout) = http_client
            .get("request-timeout")
            .and_then(|v| v.as_integer())
            .and_then(|i| u64::try_from(i).ok())
        {
            client_builder = client_builder.timeout(Duration::from_secs(timeout));
        }
        if let Some(addr) = http_client
            .get("local-address")
            .and_then(|v| v.as_str())
            .and_then(|s| IpAddr::from_str(s).ok())
        {
            client_builder = client_builder.local_address(addr);
        }
    }

    let reqwest_client = client_builder
        .build()
        .unwrap_or_else(|err| panic!("failed to create an HTTP client: {err}"));
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest_client)
        .with(TracingMiddleware::<RequestTiming>::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    SHARED_HTTP_CLIENT
        .set(client)
        .expect("failed to set an HTTP client for the application");
}

/// Constructs a request builder.
pub(crate) fn request_builder(
    resource: impl IntoUrl,
    options: impl Into<Option<Map>>,
) -> Result<RequestBuilder, BoxError> {
    let options = options.into().unwrap_or_default();
    if options.is_empty() {
        let request_builder = SHARED_HTTP_CLIENT
            .get()
            .ok_or("failed to get the global http client")?
            .request(Method::GET, resource);
        return Ok(request_builder);
    }

    let method = options
        .get("method")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(Method::GET);
    let mut request_builder = SHARED_HTTP_CLIENT
        .get()
        .ok_or("failed to get the global http client")?
        .request(method, resource);
    let mut headers = HeaderMap::new();
    if let Some(query) = options.get("query") {
        request_builder = request_builder.query(query);
    }
    if let Some(body) = options.get("body") {
        let content_type = options
            .get("content_type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        match body {
            Value::String(value) => {
                if content_type == "json" {
                    request_builder = request_builder
                        .json(value)
                        .header(header::CONTENT_TYPE, "application/json");
                } else {
                    request_builder = request_builder
                        .form(value)
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
                }
            }
            Value::Object(value) => {
                if content_type == "form" {
                    request_builder = request_builder
                        .form(value)
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
                } else {
                    request_builder = request_builder
                        .json(value)
                        .header(header::CONTENT_TYPE, "application/json");
                }
            }
            _ => tracing::warn!("unsupported body format"),
        }
    }
    if let Some(map) = options.get("headers").and_then(|v| v.as_object()) {
        for (key, value) in map {
            if let Ok(header_name) = HeaderName::try_from(key) {
                if let Some(header_value) = value.as_str().and_then(|s| s.parse().ok()) {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }
    if !headers.is_empty() {
        request_builder = request_builder.headers(headers);
    }
    if let Some(timeout) = options.get("timeout").and_then(|v| v.as_u64()) {
        request_builder = request_builder.timeout(Duration::from_millis(timeout));
    }
    Ok(request_builder)
}

/// Request timing.
struct RequestTiming;

impl ReqwestOtelSpanBackend for RequestTiming {
    fn on_request_start(request: &Request, extensions: &mut Extensions) -> Span {
        let url = request.url();
        let headers = request.headers();
        let traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
        let trace_context = traceparent.and_then(TraceContext::from_traceparent);
        extensions.insert(Instant::now());
        tracing::info_span!(
            "HTTP request",
            "otel.kind" = "client",
            "otel.name" = "zino-bot",
            "http.method" = request.method().as_str(),
            "http.scheme" = url.scheme(),
            "http.url" = remove_credentials(url).as_ref(),
            "http.request.header.traceparent" = traceparent,
            "http.request.header.tracestate" =
                headers.get("tracestate").and_then(|v| v.to_str().ok()),
            "http.response.header.traceparent" = Empty,
            "http.response.header.tracestate" = Empty,
            "http.status_code" = Empty,
            "http.client.duration" = Empty,
            "net.peer.name" = url.domain(),
            "net.peer.port" = url.port(),
            "context.request_id" = Empty,
            "context.session_id" = headers.get("session-id").and_then(|v| v.to_str().ok()),
            "context.span_id" = Empty,
            "context.trace_id" = trace_context
                .as_ref()
                .map(|ctx| Uuid::from_u128(ctx.trace_id()).to_string()),
            "context.parent_id" = trace_context
                .and_then(|ctx| ctx.parent_id())
                .map(|parent_id| format!("{parent_id:x}")),
        )
    }

    fn on_request_end(span: &Span, outcome: &Result<Response, Error>, extensions: &mut Extensions) {
        let latency_millis = extensions
            .get::<Instant>()
            .and_then(|t| u64::try_from(t.elapsed().as_millis()).ok());
        span.record("http.client.duration", latency_millis);
        span.record(
            "context.span_id",
            span.id().map(|id| format!("{:x}", id.into_u64())),
        );
        match outcome {
            Ok(response) => {
                let headers = response.headers();
                span.record(
                    "http.response.header.traceparent",
                    headers.get("traceparent").and_then(|v| v.to_str().ok()),
                );
                span.record(
                    "http.response.header.tracestate",
                    headers.get("tracestate").and_then(|v| v.to_str().ok()),
                );
                span.record(
                    "context.request_id",
                    headers.get("x-request-id").and_then(|v| v.to_str().ok()),
                );
                span.record("http.status_code", response.status().as_u16());
                tracing::info!("finished HTTP request");
            }
            Err(err) => {
                if let Error::Reqwest(err) = err {
                    span.record(
                        "http.status_code",
                        err.status().map(|status_code| status_code.as_u16()),
                    );
                }
                tracing::error!("{err}");
            }
        }
    }
}

fn remove_credentials(url: &Url) -> Cow<'_, str> {
    if !url.username().is_empty() || url.password().is_some() {
        let mut url = url.clone();
        url.set_username("")
            .and_then(|_| url.set_password(None))
            .ok();
        url.to_string().into()
    } else {
        url.as_ref().into()
    }
}

/// Shared HTTP client.
static SHARED_HTTP_CLIENT: OnceLock<ClientWithMiddleware> = OnceLock::new();