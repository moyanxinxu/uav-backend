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
use sea_orm::prelude::*;
use crate::app::AppState;
use crate::common::response::ApiResponse;
use crate::entity::incidents::ActiveModel as IncidentsActiveModel;
use crate::entity::incidents;
use crate::entity::prelude::Incidents;
use crate::common::page::{ Page, PaginationParams };
use crate::common::result::{ ApiError, ApiResult };
use crate::entity::sea_orm_active_enums::IncidentStatus;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncidentQuery {
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

async fn get_all_incidents(
    State(AppState { db }): State<AppState>,
    Query(IncidentQuery { pagination }): Query<IncidentQuery>
) -> ApiResult<ApiResponse<Page<incidents::Model>>> {
    let paginator = Incidents::find().paginate(&db, pagination.size);

    let total = paginator.num_items().await?;
    let incidents = paginator.fetch_page(pagination.page - 1).await?;

    let page = Page::from_pagination(pagination, total, incidents);
    Ok(ApiResponse::ok("ok", Some(page)))
}

// async fn get_all_available_drones(
//     State(AppState { db }): State<AppState>,
//     Query(DroneQuery { pagination }): Query<DroneQuery>
// ) -> ApiResult<ApiResponse<Page<drones::Model>>> {
//     let paginator = Drones::find()
//         .filter(drones::Column::Activate.eq(true))
//         .paginate(&db, pagination.size);

//     let total = paginator.num_items().await?;
//     let drones = paginator.fetch_page(pagination.page - 1).await?;

//     let page = Page::from_pagination(pagination, total, drones);

//     Ok(ApiResponse::ok("获取所有可用无人机成功", Some(page)))
// }

async fn get_incident(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<incidents::Model>> {
    let incident = Incidents::find_by_id(&id).one(&db).await.unwrap();
    if let Some(incident) = incident {
        Ok(ApiResponse::ok("获取事件成功", Some(incident)))
    } else {
        let msg = format!("未找到ID为{}的事件", id);
        Err(ApiError::Biz(msg))
    }
}

#[derive(Debug, Deserialize)]
struct IncidentCreateRequest {
    title: String,
    description: String,
    lat: Decimal,
    lng: Decimal,
    radius: f32,
    severity: i8,
    status: IncidentStatus,
    created_by: String,
}

async fn add_incident(
    State(AppState { db }): State<AppState>,
    Json(data): Json<IncidentCreateRequest>
) -> ApiResult<ApiResponse<()>> {
    let incident = IncidentsActiveModel {
        incident_id: ActiveValue::set(xid::new().to_string()),
        title: ActiveValue::set(data.title),
        description: ActiveValue::set(Some(data.description)),
        lat: ActiveValue::set(data.lat),
        lng: ActiveValue::set(data.lng),
        radius: ActiveValue::set(Some(data.radius)),
        severity: ActiveValue::set(Some(data.severity)),
        status: ActiveValue::set(data.status),
        created_by: ActiveValue::set(data.created_by),
        ..Default::default()
    };

    incident.insert(&db).await.unwrap();

    Ok(ApiResponse::ok("事件信息添加成功", None))
}

// 修改用户时的请求体
#[derive(Deserialize)]
struct IncidentUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lat: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lng: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<IncidentStatus>,
}

impl IncidentUpdateRequest {
    fn apply_to(&self, incident: &mut IncidentsActiveModel) {
        if let Some(ref title) = self.title && !title.is_empty() {
            incident.title = ActiveValue::set(title.clone());
        }
        if let Some(ref description) = self.description {
            incident.description = ActiveValue::set(Some(description.clone()));
        }
        if let Some(ref lat) = self.lat {
            incident.lat = ActiveValue::set(lat.clone());
        }
        if let Some(ref lng) = self.lng {
            incident.lng = ActiveValue::set(lng.clone());
        }
        if let Some(ref radius) = self.radius {
            incident.radius = ActiveValue::set(Some(*radius));
        }
        if let Some(ref status) = self.status {
            incident.status = ActiveValue::set(status.clone());
        }
    }
}

async fn update_incident(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,
    Json(data): Json<IncidentUpdateRequest>
) -> ApiResult<ApiResponse<()>> {
    let incident = Incidents::find_by_id(&id).one(&db);
    if let Some(incident) = incident.await.unwrap() {
        let mut incident = incident.into_active_model();

        data.apply_to(&mut incident);

        incident.update(&db).await.unwrap();
        Ok(ApiResponse::ok("更新事件成功", None))
    } else {
        Err(ApiError::Biz("事件未找到".to_string()))
    }
}

async fn delete_incident(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>
) -> ApiResult<ApiResponse<()>> {
    let incident = Incidents::find_by_id(&id).one(&db).await.unwrap();

    if let Some(incident) = incident {
        incident.delete(&db).await.unwrap();
        Ok(ApiResponse::ok("删除事件成功", None))
    } else {
        Err(ApiError::Biz("事件未找到".to_string()))
    }
}

// #[derive(Debug, Serialize, FromQueryResult)]
// struct IncidentStatusItem {
//     status: String,
//     count: i64,
// }

// #[derive(Debug, Serialize)]
// struct IncidentStatusResponse {
//     categories: Vec<IncidentStatusItem>,
// }

// async fn get_incident_status(State(
//     AppState { db },
// ): State<AppState>) -> ApiResult<ApiResponse<IncidentStatusResponse>> {
//     let result = Incidents::find()
//         .select_only()
//         .column(incidents::Column::Status)
//         .column_as(Expr::col(incidents::Column::IncidentId).count(), "count")
//         .group_by(incidents::Column::Status)
//         .into_model::<IncidentStatusItem>()
//         .all(&db).await?;

//     Ok(ApiResponse::ok("获取事件状态分布成功", Some(IncidentStatusResponse { categories: result })))
// }

pub fn create_incident_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_incidents))
        .route("/", post(add_incident))
        .route("/{id}", get(get_incident))
        .route("/{id}", put(update_incident))
        .route("/{id}", delete(delete_incident))
    // .route("/available", get(get_all_available_incidents))
    //.route("/status", get(get_incident_status))
}
