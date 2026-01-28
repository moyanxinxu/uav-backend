use axum::Router;
use axum::extract::{ Query, State };
use axum::routing::{ get };

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use crate::app::AppState;
use crate::common::page::{ Page, PaginationParams };
use crate::common::response::ApiResponse;
use crate::common::result::ApiResult;
use crate::entity::logs;
use crate::entity::prelude::Logs;
use crate::entity::logs::ActiveModel as LogsActiveModel;
use crate::entity::sea_orm_active_enums::LogType;

async fn get_all_logs(
    State(AppState { db }): State<AppState>,
    Query(pagination): Query<PaginationParams>
) -> ApiResult<ApiResponse<Page<logs::Model>>> {
    let paginator = Logs::find().paginate(&db, pagination.size);

    let total = paginator.num_items().await?;
    let logs = paginator.fetch_page(pagination.page - 1).await?;

    let page = Page::from_pagination(pagination, total, logs);

    Ok(ApiResponse::ok("ok", Some(page)))
}

pub struct LogManager;

impl LogManager {
    pub async fn create_log(
        db: &DatabaseConnection,
        log_type: LogType,
        message: String
    ) -> ApiResult<()> {
        let log = LogsActiveModel {
            log_id: ActiveValue::NotSet,
            log_type: ActiveValue::set(log_type),
            message: ActiveValue::set(message),
            created_at: ActiveValue::NotSet,
        };

        log.insert(db).await?;
        Ok(())
    }

    pub fn info() -> LogType {
        LogType::Info
    }

    pub fn error() -> LogType {
        LogType::Error
    }

    pub fn warning() -> LogType {
        LogType::Warn
    }
}

pub fn create_logs_router() -> Router<AppState> {
    Router::new().route("/", get(get_all_logs))
}
