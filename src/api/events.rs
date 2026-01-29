use axum::extract::{ Path, Query, State, Json };
use axum::Router;
use axum::routing::{ delete, get, post, put };
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    EntityTrait,
    IntoActiveModel,
    ModelTrait,
    PaginatorTrait,
};

use crate::app::AppState;
use crate::common::response::ApiResponse;
use crate::entity::events::ActiveModel as EventsActiveModel;
use crate::entity::events;
use crate::entity::prelude::Events;
use crate::common::page::{ Page, PaginationParams };
use crate::common::result::{ ApiError, ApiResult };
use crate::entity::sea_orm_active_enums::EventType;
use serde::{ Deserialize };

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
}

async fn get_all_events(
    State(AppState { db }): State<AppState>,
    Query(EventQuery { pagination }): Query<EventQuery>
) -> ApiResult<ApiResponse<Page<events::Model>>> {
    let paginator = Events::find().paginate(&db, pagination.size);
    let total = paginator.num_items().await?;
    let events = paginator.fetch_page(pagination.page - 1).await?;

    let page = Page::from_pagination(pagination, total, events);

    Ok(ApiResponse::ok("获取事件列表成功", Some(page)))
}

async fn get_event(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<events::Model>> {
    let event = Events::find_by_id(&id).one(&db).await.unwrap();

    if let Some(event) = event {
        Ok(ApiResponse::ok("获取事件成功", Some(event)))
    } else {
        let msg = format!("Event id {} not found", id);
        Err(ApiError::Biz(msg))
    }
}

#[derive(Debug, Deserialize)]
struct EventCreateRequest {
    mission_id: String,
    event_type: EventType,
    message: String,
}

async fn add_event(
    State(AppState { db }): State<AppState>,
    Json(data): Json<EventCreateRequest>
) -> ApiResult<ApiResponse<()>> {
    let event = EventsActiveModel {
        event_id: ActiveValue::set(xid::new().to_string()),
        mission_id: ActiveValue::set(data.mission_id),
        event_type: ActiveValue::set(data.event_type),
        message: ActiveValue::set(Some(data.message)),
        ..Default::default()
    };

    event.insert(&db).await.unwrap();

    Ok(ApiResponse::ok("事件信息添加成功", None))
}

// 修改用户时的请求体
#[derive(Deserialize)]
struct EventUpdateRequest {
    event_type: Option<EventType>,
    message: Option<String>,
}

impl EventUpdateRequest {
    fn apply_to(&self, event: &mut EventsActiveModel) {
        if let Some(ref event_type) = self.event_type {
            event.event_type = ActiveValue::set(event_type.clone());
        }
        if let Some(ref message) = self.message {
            event.message = ActiveValue::set(Some(message.clone()));
        }
    }
}

async fn update_event(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(data): Json<EventUpdateRequest>
) -> ApiResult<ApiResponse<()>> {
    let event = Events::find_by_id(&id).one(&db);
    if let Some(event) = event.await.unwrap() {
        let mut event = event.into_active_model();

        data.apply_to(&mut event);

        event.update(&db).await.unwrap();
        Ok(ApiResponse::ok("更新事件成功", None))
    } else {
        Err(ApiError::Biz("事件未找到".to_string()))
    }
}

async fn delete_event(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<()>> {
    let event = Events::find_by_id(&id).one(&db).await.unwrap();

    if let Some(event) = event {
        event.delete(&db).await.unwrap();
        Ok(ApiResponse::ok("删除事件成功", None))
    } else {
        Err(ApiError::Biz("事件未找到".to_string()))
    }
}

pub fn create_event_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_events))
        .route("/", post(add_event))
        .route("/{id}", get(get_event))
        .route("/{id}", put(update_event))
        .route("/{id}", delete(delete_event))
}
