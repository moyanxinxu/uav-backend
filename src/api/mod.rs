mod user;
mod drone;
mod mission;
mod logs;
mod events;
mod incident;

use crate::api::drone::create_drone_router;
use crate::api::events::create_event_router;
use crate::api::logs::create_logs_router;
use crate::api::mission::create_mission_router;
use crate::api::user::create_user_router;
use crate::api::incident::create_incident_router;
use crate::app::AppState;
use crate::common::result::{ ApiError, ApiResult };
use axum::Router;

use axum::http::Method;
use tower_http::cors::{ Any, CorsLayer };

pub fn create_overall_router() -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new().nest(
        "/api",
        Router::new()
            .nest("/users", create_user_router())
            .nest("/drones", create_drone_router())
            .nest("/missions", create_mission_router())
            .nest("/logs", create_logs_router())
            .nest("/events", create_event_router())
            .nest("/incidents", create_incident_router())
            .fallback(async || -> ApiResult<()> { Err(ApiError::NotFound) })
            .layer(cors)
    )
}
