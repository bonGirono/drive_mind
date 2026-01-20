use apalis::prelude::*;
use apalis_cron::{CronStream, Schedule};
use axum::response::IntoResponse;
use sea_orm::DatabaseConnection;
use std::net::SocketAddr;
use std::str::FromStr;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

mod entities;
mod migrations;
mod models;
mod rest;
mod tasks;
mod utils;

#[derive(Clone)]
struct AppContext {
    db: DatabaseConnection,
}

#[derive(OpenApi)]
#[openapi(
    components(schemas()),
    security(
        ("jwt_token" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

// Add Bearer Auth to components
pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "jwt_token",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT") // optional
                    .build(),
            ),
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let server_config = utils::config::ServerConfig::load();
    let db_config = utils::config::DBConfig::load();

    let db = db_config.connect().await;
    let state = AppContext { db };
    let state_clone = state.clone();
    let schedule = Schedule::from_str("0 * * * * *").unwrap();
    let worker = WorkerBuilder::new("morning-cereal")
        .retry(apalis::layers::retry::RetryPolicy::retries(5))
        .data(state_clone)
        .backend(CronStream::new(schedule))
        .build_fn(tasks::scheduled_task);

    tokio::spawn(async move {
        worker.run().await;
    });

    let addr: SocketAddr = server_config.get_addr();

    tracing::info!(message = "Starting server.", %addr);
    tracing::info!("Check openapi docs: http://{addr}/scalar");

    let (app, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(healthcheck))
        .merge(rest::auth::routes())
        .merge(rest::users::routes())
        .merge(rest::images::routes())
        .merge(rest::topics::routes())
        .merge(rest::lessons::routes())
        .merge(rest::questions::routes())
        .merge(rest::answers::routes())
        .merge(rest::user_favorite_questions::routes())
        .merge(rest::categories::routes())
        .merge(rest::question_categories::routes())
        .merge(rest::tests::routes())
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::default().include_headers(true),
            ),
        )
        .layer(axum::Extension(server_config))
        .layer(tower_http::cors::CorsLayer::permissive())
        .route_layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
        .split_for_parts();

    let app = app.merge(Scalar::with_url("/scalar", api.clone()));
    let app = app.merge(utoipa_rapidoc::RapiDoc::with_url(
        "/rapid",
        "/api-docs/openapi.json",
        api,
    ));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let shutdown_signal = async {
        tokio::signal::ctrl_c().await.unwrap();
        println!("Ctrl+C received — shutting down...");
    };

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal)
    .await
    .unwrap();

    Ok(())
}

#[utoipa::path(
    get,
    tag = "Healthcheck",
    path = "/healthcheck",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    ),
    security()
)]
async fn healthcheck(
    axum::extract::State(state): axum::extract::State<AppContext>,
) -> impl IntoResponse {
    if let Ok(_) = state.db.ping().await {
        return ("Ok").into_response();
    }

    ("DbErr").into_response()
}
