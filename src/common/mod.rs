pub mod response;
pub mod result;
pub mod page;
pub mod logger;

mod server;
use server::ServerConfig;

const HOST: &str = "0.0.0.0";
const PORT: u16 = 5001;

pub fn host() -> String {
    HOST.to_string()
}

pub fn port() -> u16 {
    PORT
}

pub struct AppConfig {
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn new(server: ServerConfig) -> Self {
        AppConfig { server }
    }

    pub fn url(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

pub fn load_app_config() -> AppConfig {
    let server_config = ServerConfig::new(host(), port());

    AppConfig::new(server_config)
}
