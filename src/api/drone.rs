use axum::extract::{ Path, Query, State, Json };
use axum::Router;
use axum::routing::{ delete, get, post, put };
use sea_orm::{
    ActiveModelTrait,
    ActiveValue,
    ColumnTrait,
    EntityTrait,
    IntoActiveModel,
    ModelTrait,
    PaginatorTrait,
    QueryFilter,
    FromQueryResult,
    QuerySelect,
};
use sea_orm::prelude::Expr;
use crate::app::AppState;
use crate::common::response::ApiResponse;
use crate::entity::drones::ActiveModel as DronesActiveModel;
use crate::entity::drones;
use crate::entity::prelude::Drones;
use crate::common::page::{ Page, PaginationParams };
use crate::common::result::{ ApiError, ApiResult };
use crate::entity::sea_orm_active_enums::Status;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DroneQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
}

// struct DroneResponse {
//     id: String,
//     name: String,
//     model: String,
//     status: Status,
//     battery_level: Option<i32>,
// }

async fn get_all_drones(
    State(AppState { db }): State<AppState>,
    Query(DroneQuery { pagination }): Query<DroneQuery>
) -> ApiResult<ApiResponse<Page<drones::Model>>> {
    let paginator = Drones::find().paginate(&db, pagination.size);

    let total = paginator.num_items().await?;
    let drones = paginator.fetch_page(pagination.page - 1).await?;

    let page = Page::from_pagination(pagination, total, drones);

    Ok(ApiResponse::ok("ok", Some(page)))
}

async fn get_all_available_drones(
    State(AppState { db }): State<AppState>,
    Query(DroneQuery { pagination }): Query<DroneQuery>
) -> ApiResult<ApiResponse<Page<drones::Model>>> {
    let paginator = Drones::find()
        .filter(drones::Column::Activate.eq(true))
        .paginate(&db, pagination.size);

    let total = paginator.num_items().await?;
    let drones = paginator.fetch_page(pagination.page - 1).await?;

    let page = Page::from_pagination(pagination, total, drones);

    Ok(ApiResponse::ok("获取所有可用无人机成功", Some(page)))
}

async fn get_drone(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<drones::Model>> {
    let drone = Drones::find_by_id(&id).one(&db).await.unwrap();

    if let Some(drone) = drone {
        Ok(ApiResponse::ok("ok", Some(drone)))
    } else {
        let msg = format!("Drone id {} not found", id);
        Err(ApiError::Biz(msg))
    }
}

#[derive(Debug, Deserialize)]
struct DroneCreateRequest {
    name: String,
    model: String,
    status: Status,
    battery: u8,
}

async fn add_drone(
    State(AppState { db }): State<AppState>,
    Json(data): Json<DroneCreateRequest>
) -> ApiResult<ApiResponse<()>> {
    let drone = DronesActiveModel {
        drone_id: ActiveValue::set(xid::new().to_string()),
        name: ActiveValue::set(data.name),
        model: ActiveValue::set(data.model),
        status: ActiveValue::set(data.status),
        battery: ActiveValue::set(data.battery),
        // created_at: ActiveValue::set(Some(Utc::now().naive_utc())),
        ..Default::default()
    };

    drone.insert(&db).await.unwrap();

    Ok(ApiResponse::ok("无人机信息添加成功", None))
}

// 修改用户时的请求体
#[derive(Deserialize)]
struct DroneUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    battery: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activate: Option<i8>,
}

impl DroneUpdateRequest {
    fn apply_to(&self, drone: &mut DronesActiveModel) {
        if let Some(ref name) = self.name && !name.is_empty() {
            drone.name = ActiveValue::set(name.clone());
        }
        if let Some(ref model) = self.model {
            drone.model = ActiveValue::set(model.clone());
        }
        if let Some(ref status) = self.status {
            drone.status = ActiveValue::set(status.clone());
        }
        if let Some(ref battery) = self.battery {
            drone.battery = ActiveValue::set(*battery);
        }
        if let Some(ref activate) = self.activate {
            drone.activate = ActiveValue::set(*activate);
        }
    }
}

async fn update_drone(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(data): Json<DroneUpdateRequest>
) -> ApiResult<ApiResponse<()>> {
    let drone = Drones::find_by_id(&id).one(&db);
    if let Some(drone) = drone.await.unwrap() {
        let mut drone = drone.into_active_model();

        data.apply_to(&mut drone);

        drone.update(&db).await.unwrap();
        Ok(ApiResponse::ok("更新无人机成功", None))
    } else {
        Err(ApiError::Biz("无人机未找到".to_string()))
    }
}

async fn delete_drone(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<()>> {
    let drone = Drones::find_by_id(&id).one(&db).await.unwrap();

    if let Some(drone) = drone {
        drone.delete(&db).await.unwrap();
        Ok(ApiResponse::ok("删除无人机成功", None))
    } else {
        Err(ApiError::Biz("无人机未找到".to_string()))
    }
}

#[derive(Debug, Serialize, FromQueryResult)]
struct DroneStatusItem {
    status: String,
    count: i64,
}

#[derive(Debug, Serialize)]
struct DroneStatusResponse {
    categories: Vec<DroneStatusItem>,
}

async fn get_drone_status(State(
    AppState { db },
): State<AppState>) -> ApiResult<ApiResponse<DroneStatusResponse>> {
    let result = Drones::find()
        .select_only()
        .column(drones::Column::Status)
        .column_as(Expr::col(drones::Column::DroneId).count(), "count")
        .group_by(drones::Column::Status)
        .into_model::<DroneStatusItem>()
        .all(&db).await?;

    Ok(ApiResponse::ok("获取无人机状态分布成功", Some(DroneStatusResponse { categories: result })))
}

pub fn create_drone_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_drones))
        .route("/", post(add_drone))
        .route("/{id}", get(get_drone))
        .route("/{id}", put(update_drone))
        .route("/{id}", delete(delete_drone))
        .route("/available", get(get_all_available_drones))
        .route("/status", get(get_drone_status))
}
