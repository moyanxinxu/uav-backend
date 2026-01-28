use axum::extract::{ Json, Path, Query, State };
use axum::Router;

use axum::routing::{ delete, get, post, put };
use sea_orm::{ QuerySelect };

use serde::{ Deserialize, Serialize };

use crate::api::logs::LogManager;
use crate::app::AppState;

use crate::common::page::{ Page, PaginationParams };
use crate::common::response::ApiResponse;
use crate::common::result::{ ApiError, ApiResult };
use crate::entity::prelude::Users;
use crate::entity::sea_orm_active_enums::Role;
use crate::entity::users;
use crate::entity::users::ActiveModel as UsersActiveModel;
use sea_orm::entity::prelude::*;
use sea_orm::{ ActiveValue, FromQueryResult, IntoActiveModel };

// 创建用户时的请求体
#[derive(Deserialize)]
pub struct UserCreateRequest {
    name: String,
    password: String,
    role: Role,
}

// 修改用户时的请求体
#[derive(Deserialize, Debug)]
pub struct UserUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<Role>,
}

impl UserUpdateRequest {
    fn apply_to(&self, user: &mut UsersActiveModel) {
        if let Some(ref name) = self.name && !name.is_empty() {
            user.name = ActiveValue::set(name.clone());
        }
        if let Some(ref role) = self.role {
            user.role = ActiveValue::set(role.clone());
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct UserResponse {
    user_id: String,
    name: String,
    role: Role,
}

async fn get_all_users(
    State(AppState { db }): State<AppState>,
    Query(UserQuery { pagination }): Query<UserQuery>
) -> ApiResult<ApiResponse<Page<UserResponse>>> {
    let paginator = Users::find()
        .select_only()
        .column(users::Column::UserId)
        .column(users::Column::Name)
        .column(users::Column::Role)
        // .order_by_desc(users::Column::CreatedAt)
        .into_model::<UserResponse>()
        .paginate(&db, pagination.size);

    let total = paginator.num_items().await?;

    match paginator.fetch_page(pagination.page - 1).await {
        Ok(users) => {
            let page = Page::from_pagination(pagination, total, users);
            LogManager::create_log(&db, LogManager::info(), format!("获取所有用户成功")).await?;
            Ok(ApiResponse::ok("查找用户成功", Some(page)))
        }
        Err(e) => {
            LogManager::create_log(&db, LogManager::error(), format!("获取用户失败")).await?;
            Err(ApiError::Biz(format!("查找所有用户失败：{}", e)))
        }
    }
}

async fn add_user(
    State(AppState { db }): State<AppState>,
    Json(data): Json<UserCreateRequest>
) -> ApiResult<ApiResponse<()>> {
    let user_name = data.name.clone();

    let user = UsersActiveModel {
        user_id: ActiveValue::set(xid::new().to_string()),
        name: ActiveValue::set(data.name),
        password: ActiveValue::set(data.password),
        role: ActiveValue::set(data.role),
        ..Default::default()
    };

    match user.insert(&db).await {
        Ok(_) => {
            LogManager::create_log(
                &db,
                LogManager::info(),
                format!("创建用户<{}>成功", user_name)
            ).await?;
            Ok(ApiResponse::ok("创建用户成功", None))
        }
        Err(e) => {
            LogManager::create_log(
                &db,
                LogManager::error(),
                format!("创建用户<{}>失败: {}", user_name, e)
            ).await?;
            Err(ApiError::Biz(format!("创建用户失败: {}", e)))
        }
    }
}

async fn get_user(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<UserResponse>> {
    let user = Users::find()
        .select_only()
        .column(users::Column::UserId)
        .column(users::Column::Name)
        .column(users::Column::Role)
        .filter(users::Column::UserId.eq(&id))
        .into_model::<UserResponse>()
        .one(&db).await
        .unwrap();

    if let Some(user) = user {
        LogManager::create_log(&db, LogManager::info(), format!("获取用户<{}>", user.name)).await?;
        Ok(ApiResponse::ok("获取用户成功", Some(user)))
    } else {
        LogManager::create_log(&db, LogManager::info(), format!("未找到该用户<{}>", id)).await?;
        Err(ApiError::Biz("未找到该用户".to_string()))
    }
}

async fn delete_user(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<()>> {
    let user = Users::find_by_id(&id).one(&db).await.unwrap();

    if let Some(user) = user {
        let user_name = user.name.clone();

        user.delete(&db).await.unwrap();

        LogManager::create_log(&db, LogManager::info(), format!("删除用户<{}>", user_name)).await?;
        Ok(ApiResponse::ok("删除用户成功", None))
    } else {
        LogManager::create_log(&db, LogManager::info(), format!("未找到该用户<{}>", id)).await?;
        Err(ApiError::Biz("未找到该用户".to_string()))
    }
}

async fn update_user(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(data): Json<UserUpdateRequest>
) -> ApiResult<ApiResponse<()>> {
    let user = Users::find_by_id(&id).one(&db).await.unwrap();

    if let Some(user) = user {
        let mut user = user.into_active_model();

        data.apply_to(&mut user);

        user.update(&db).await.unwrap();

        LogManager::create_log(
            &db,
            LogManager::info(),
            format!("更新用户参数为：{:?}", data)
        ).await?;

        Ok(ApiResponse::ok("更新用户成功", None))
    } else {
        LogManager::create_log(&db, LogManager::info(), format!("未找到该用户<{}>", id)).await?;
        Err(ApiError::Biz("未找到该用户".to_string()))
    }
}

pub fn create_user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_users))
        .route("/", post(add_user))
        .route("/{id}", get(get_user))
        .route("/{id}", put(update_user))
        .route("/{id}", delete(delete_user))
}
