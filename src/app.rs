use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        AppState { db }
    }

    // pub fn db(&self) -> &DatabaseConnection {
    //     &self.db
    // }
}
