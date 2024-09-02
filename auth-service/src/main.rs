use std::sync::Arc;
use sqlx::{Executor, PgPool};
use sqlx::postgres::PgPoolOptions;
use auth_service::{Application, AppState, MockEmailClient, get_postgres_pool, PostgresUserStore, get_redis_client, RedisBannedTokenStore, RedisTwoFACodeStore};
use tokio::sync::RwLock;
use uuid::Uuid;
use auth_service::utils::constants::{prod, DATABASE_URL, REDIS_HOST_NAME};

#[tokio::main]
async fn main() {

    let pg_pool = configure_postgresql().await;

    // let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let user_store =
        Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let redis_conn = Arc::new(RwLock::new(configure_redis()));
    // let banned_token_store =
    //     Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let banned_token_store =
        Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn.clone())));
    // let two_fa_code_store =
    //     Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));

    let two_fa_code_store =
        Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn)));

    let email_client =
        Arc::new(MockEmailClient);

    let app_state = AppState::new(user_store, banned_token_store, two_fa_code_store,
                                  email_client);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
