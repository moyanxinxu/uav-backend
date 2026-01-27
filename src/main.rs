mod api;
mod app;
mod common;
mod entity;

use crate::api::create_overall_router;
use crate::app::AppState;
use sea_orm::Database;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // let db = Database::connect("mysql://moyanxinxu:moyanxinxu@database:3306/uav")
    let db = Database::connect("sqlite://scipts/uav.db")
        .await
        .unwrap();

    let app_config = common::load_app_config();
    let app_state = AppState::new(db);

    let listener = TcpListener::bind(app_config.url()).await.unwrap();
    let router = create_overall_router().with_state(app_state);
    println!(
        "Server running at http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, router).await.unwrap();
}
