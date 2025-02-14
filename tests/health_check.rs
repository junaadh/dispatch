use std::net::TcpListener;

use dispatch::configuration::Settings;
use sqlx::{Connection, PgConnection};

fn spawn_app() -> String {
    let listener = TcpListener::bind("0.0.0.0:0")
        .expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = dispatch::startup::run(listener)
        .expect("Failed to bind address");

    tokio::spawn(server);
    format!("http://0.0.0.0:{port}")
}

#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &addr))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let config = Settings::get_conf().expect("failed to read config");
    let connection_string = config.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("fauked to connect to postgres");
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=urusula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved =
        sqlx::query!("SELECT email, name FROM subscriptions",)
            .fetch_one(&mut connection)
            .await
            .expect("failed to fetch saved subscriptions");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app_address))
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
