use std::net::TcpListener;

use dispatch::{configuration::Settings, startup::run};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let conf = Settings::get_conf().expect("Failed to read config");
    let connection_pool =
        PgPool::connect(&conf.database.connection_string())
            .await
            .expect("failed to connect to postgres");
    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        conf.application_port
    ))?;
    run(listener, connection_pool)?.await
}
