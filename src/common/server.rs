pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn new(host: String, port: u16) -> Self {
        ServerConfig { host, port }
    }
}
