mod user;

use crate::api::user::create_user_router;
use crate::app::AppState;
use crate::common::result::{ApiError, ApiResult};
use axum::Router;

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub fn create_overall_router() -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new().nest(
        "/api",
        Router::new()
            .nest("/users", create_user_router())
            .fallback(async || -> ApiResult<()> { Err(ApiError::NotFound) })
            .layer(cors),
    )
}
