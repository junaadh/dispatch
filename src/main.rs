use std::net::TcpListener;

use dispatch::{configuration::Settings, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let conf = Settings::get_conf().expect("Failed to read config");
    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        conf.application_port
    ))?;
    run(listener)?.await
}
