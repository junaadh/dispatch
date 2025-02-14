use dispatch::configuration::{DatabaseSettings, Settings};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("0.0.0.0:0")
        .expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://0.0.0.0:{port}");

    let mut config =
        Settings::get_conf().expect("failed to read config");
    config.database.database_name = uuid::Uuid::new_v4().to_string();
    let conn_pool = configure_db(&config.database).await;

    let server = dispatch::startup::run(listener, conn_pool.clone())
        .expect("Failed to bind address");

    tokio::spawn(server);
    TestApp {
        address: addr,
        db_pool: conn_pool,
    }
}

pub async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db())
            .await
            .expect("failed to connect to postgres");
    connection
        .execute(
            format!(r#"CREATE DATABASE "{}";"#, config.database_name)
                .as_str(),
        )
        .await
        .expect("failed to create database");

    let connection_pool =
        PgPool::connect(&config.connection_string())
            .await
            .expect("failed to connect to postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=urusula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved =
        sqlx::query!("SELECT email, name FROM subscriptions",)
            .fetch_one(&app.db_pool)
            .await
            .expect("failed to fetch saved subscriptions");

    assert_eq!(saved.email, "urusula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded",
            )
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail with 400 Bad Request when the payload was {}",
            error_msg
        );
    }
}
