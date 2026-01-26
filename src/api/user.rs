use axum::Router;
use axum::extract::{Json, Path, State};

use axum::routing::{delete, get, post};
use sea_orm::QuerySelect;

use serde::{Deserialize, Serialize};

use crate::app::AppState;

use crate::common::response::ApiResponse;
use crate::common::result::{ApiError, ApiResult};
use crate::entity::prelude::Users;
use crate::entity::sea_orm_active_enums::Role;
use crate::entity::users;
use crate::entity::users::ActiveModel as UsersActiveModel;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, FromQueryResult, IntoActiveModel};

// 创建用户时的请求体
#[derive(Deserialize)]
pub struct UserCreateRequest {
    name: String,
    password_hash: String,
    role: Role,
}

// 修改用户时的请求体
#[derive(Deserialize)]
pub struct UserUpdateRequest {
    name: String,
    role: Role,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct UserResponse {
    id: String,
    name: String,
    role: Role,
}

async fn get_all_users(
    State(AppState { db }): State<AppState>,
) -> ApiResult<ApiResponse<Vec<UserResponse>>> {
    let users = Users::find()
        .select_only()
        .column(users::Column::Id)
        .column(users::Column::Name)
        .column(users::Column::Role)
        .into_model::<UserResponse>()
        .all(&db)
        .await
        .unwrap();

    Ok(ApiResponse::ok("ok", Some(users)))
}

async fn add_user(
    State(AppState { db }): State<AppState>,
    Json(data): Json<UserCreateRequest>,
) -> ApiResult<ApiResponse<()>> {
    let user = UsersActiveModel {
        id: ActiveValue::set(xid::new().to_string()),
        name: ActiveValue::set(data.name),
        password_hash: ActiveValue::set(data.password_hash),
        role: ActiveValue::set(data.role),
        ..Default::default()
    };

    user.insert(&db).await.unwrap();

    Ok(ApiResponse::ok("Created user successfully", None))
}

async fn get_user(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<ApiResponse<UserResponse>> {
    let user = Users::find()
        .select_only()
        .column(users::Column::Id)
        .column(users::Column::Name)
        .column(users::Column::Role)
        .filter(users::Column::Id.eq(&id))
        .into_model::<UserResponse>()
        .one(&db)
        .await
        .unwrap();

    if let Some(user) = user {
        Ok(ApiResponse::ok("ok", Some(user)))
    } else {
        let msg = format!("User id {} not found", id);
        Err(ApiError::Biz(msg))
    }
}

async fn delete_user(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<ApiResponse<()>> {
    let user = Users::find_by_id(&id).one(&db).await.unwrap();

    if let Some(user) = user {
        user.delete(&db).await.unwrap();
        Ok(ApiResponse::ok("Deleted user successfully", None))
    } else {
        Err(ApiError::Biz("User not found".to_string()))
    }
}

async fn update_user(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(data): Json<UserUpdateRequest>,
) -> ApiResult<ApiResponse<()>> {
    let user = Users::find_by_id(&id).one(&db).await.unwrap();

    if let Some(user) = user {
        let mut user = user.into_active_model();
        user.name = ActiveValue::set(data.name);
        user.role = ActiveValue::set(data.role);

        user.update(&db).await.unwrap();
        Ok(ApiResponse::ok("Updated user successfully", None))
    } else {
        Err(ApiError::Biz("User not found".to_string()))
    }
}

pub fn create_user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_users))
        .route("/", post(add_user))
        .route("/{id}", get(get_user))
        .route("/{id}", post(update_user))
        .route("/{id}", delete(delete_user))
}
