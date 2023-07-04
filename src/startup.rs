use axum::{
    extract::MatchedPath,
    response::Response,
    routing::{get, IntoMakeService},
    Extension, Router,
};
use hyper::{server::conn::AddrIncoming, Body, Request, Server};
use sqlx::PgPool;
use std::{net::TcpListener, time::Duration};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{field, Span};

use crate::routes::health_check;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

pub fn run(listener: TcpListener, pool: PgPool) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let app_state = AppState { pool };

    let svc = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str)
                        .unwrap_or("");

                    tracing::info_span!(
                        "REQUEST",
                        request_id = uuid::Uuid::new_v4().to_string(),
                        method = request.method().to_string(),
                        %matched_path,
                        status_code = field::Empty
                    )
                })
                .on_response(|response: &Response, _latency: Duration, span: &Span| {
                    span.record("status_code", response.status().as_u16());
                }),
        )
        .layer(Extension(app_state));

    let app = Router::new()
        .route("/health_check", get(health_check))
        .layer(svc);

    axum::Server::from_tcp(listener)
        .expect("Failed to bind server")
        .serve(app.into_make_service())
}
