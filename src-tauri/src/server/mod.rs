pub mod openapi;
pub mod overlays;
pub mod routes;
pub mod ws;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, instrument};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (router, api) = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .route("/api/status", axum::routing::get(routes::get_status))
        .route(
            "/api/devices/scan",
            axum::routing::get(routes::scan_devices),
        )
        .route(
            "/api/devices/connect",
            axum::routing::post(routes::connect_device),
        )
        .route(
            "/api/devices/disconnect",
            axum::routing::post(routes::disconnect_device),
        )
        .route("/api/config", axum::routing::get(routes::get_config))
        .route("/api/config", axum::routing::put(routes::update_config))
        .route("/api/ws", axum::routing::get(ws::ws_handler))
        .route(
            "/overlay",
            axum::routing::get(overlays::standard::overlay_page),
        )
        .route(
            "/overlay/chart",
            axum::routing::get(overlays::chart::overlay_chart_page),
        )
        .route(
            "/overlay/ring",
            axum::routing::get(overlays::ring::overlay_ring_page),
        )
        .split_for_parts();

    let scalar: Router = Scalar::with_url("/api/docs", api.clone()).into();

    router
        .with_state(state)
        .merge(scalar)
        .route(
            "/api/openapi.json",
            axum::routing::get(move || async { axum::Json(api) }),
        )
        .layer(cors)
}

#[instrument(skip(state), fields(%host, %port))]
pub async fn start_server(state: AppState, host: &str, port: u16) {
    let app = create_router(state);
    let addr = format!("{}:{}", host, port);
    info!(addr = %addr, "starting http/websocket server");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind server address");

    if let Err(e) = axum::serve(listener, app).await {
        error!(error = %e, "server error");
    }
}
