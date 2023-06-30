use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};

// tokio::test equivalent to tokio::main
// but for running tests
// test implementation here is decoupled from App implementation
#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// only this depends on our app
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut config = get_configuration().expect("Failed to read configuration.");

    config.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&config.database).await;

    let server =
        zero2prod::startup::run(listener, db_pool.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // set up
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    // action
    let body = "name=Frodo%20Baggins&email=frodo.baggins@40gmail.com";
    let res = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "frodo.baggins@40gmail.com");
    assert_eq!(saved.name, "Frodo Baggins");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin@", "missing the email"),
        ("email=ursula_le_guin@40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let res = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            res.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}.",
            error_message
        )
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_no_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");

    connection_pool
}
