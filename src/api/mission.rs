use axum::extract::{ Json, Path, Query, State };
use axum::Router;

use axum::routing::{ delete, get, post, put };

use serde::{ Deserialize, Serialize };

use crate::app::AppState;

use crate::common::page::{ Page, PaginationParams };
use crate::common::response::ApiResponse;
use crate::common::result::{ ApiError, ApiResult };
use crate::entity::prelude::Missions;
use crate::entity::sea_orm_active_enums::Status;
use crate::entity::missions;
use crate::entity::missions::ActiveModel as MissionsActiveModel;
use sea_orm::entity::prelude::*;
use sea_orm::{ ActiveValue, IntoActiveModel };

// use Decimal;

// #[derive(Serialize, Debug)]
// struct MissionResponse {
//     mission_id: String,
//     user_id: String,
//     target_lat: Decimal,
//     target_lng: Decimal,

//     status: Status,
//     created_at: DateTimeUtc,
//     started_at: Option<DateTimeUtc>,
//     completed_at: Option<DateTimeUtc>,
// }

// 修改任务时的请求体
#[derive(Deserialize)]
pub struct MissionUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    target_lat: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_lng: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed_at: Option<DateTime>,
}

impl MissionUpdateRequest {
    fn apply_to(&self, mission: &mut MissionsActiveModel) {
        if let Some(ref target_lat) = self.target_lat {
            mission.target_lat = ActiveValue::set(Some(target_lat.clone()));
        }
        if let Some(ref target_lng) = self.target_lng {
            mission.target_lng = ActiveValue::set(Some(target_lng.clone()));
        }
        if let Some(ref status) = self.status {
            mission.status = ActiveValue::set(status.clone());
        }

        if let Some(ref started_at) = self.started_at {
            mission.started_at = ActiveValue::set(Some(started_at.clone()));
        }
        if let Some(ref completed_at) = self.completed_at {
            mission.completed_at = ActiveValue::set(Some(completed_at.clone()));
        }
    }
}

async fn get_all_mission(
    State(AppState { db }): State<AppState>,
    Query(pagination): Query<PaginationParams>
) -> ApiResult<ApiResponse<Page<missions::Model>>> {
    let paginator = Missions::find().paginate(&db, pagination.size);

    let total = paginator.num_items().await?;
    let missions = paginator.fetch_page(pagination.page - 1).await?;

    let page = Page::from_pagination(pagination, total, missions);

    Ok(ApiResponse::ok("ok", Some(page)))
}

async fn get_mission(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<missions::Model>> {
    let mission = Missions::find_by_id(&id).one(&db).await.unwrap();

    if let Some(mission) = mission {
        Ok(ApiResponse::ok("ok", Some(mission)))
    } else {
        let msg = format!("任务ID<{}>未找到", id);
        Err(ApiError::Biz(msg))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MissionCreateRequest {
    user_id: String,
    drone_id: String,
    target_lat: Decimal,
    target_lng: Decimal,
}

async fn add_mission(
    State(AppState { db }): State<AppState>,
    Json(data): Json<MissionCreateRequest>
) -> ApiResult<ApiResponse<()>> {
    let mission = MissionsActiveModel {
        mission_id: ActiveValue::set(xid::new().to_string()),
        user_id: ActiveValue::set(data.user_id),
        drone_id: ActiveValue::set(data.drone_id),
        target_lat: ActiveValue::set(Some(data.target_lat)),
        target_lng: ActiveValue::set(Some(data.target_lng)),
        status: ActiveValue::set(Status::Idle),
        ..Default::default()
    };

    mission.insert(&db).await.unwrap();

    Ok(ApiResponse::ok("创建任务成功", None))
}

async fn delete_mission(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<()>> {
    let data = Missions::find_by_id(&id).one(&db).await.unwrap();

    if let Some(data) = data {
        data.delete(&db).await.unwrap();
        Ok(ApiResponse::ok("删除任务成功", None))
    } else {
        let msg = format!("未找到该任务");
        Err(ApiError::Biz(msg))
    }
}

async fn update_mission(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(data): Json<MissionUpdateRequest>
) -> ApiResult<ApiResponse<()>> {
    let mission = Missions::find_by_id(id).one(&db).await.unwrap();

    if let Some(mission) = mission {
        let mut mission = mission.into_active_model();

        data.apply_to(&mut mission);

        mission.update(&db).await.unwrap();
        Ok(ApiResponse::ok("更新任务成功", None))
    } else {
        let msg = format!("未找到该任务");
        Err(ApiError::Biz(msg))
    }
}

pub fn create_mission_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_mission))
        .route("/", post(add_mission))
        .route("/{id}", get(get_mission))
        .route("/{id}", put(update_mission))
        .route("/{id}", delete(delete_mission))
}
