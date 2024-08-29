use std::sync::Arc;
use sqlx::PgPool;
use auth_service::{Application, AppState, HashmapUserStore, HashsetBannedTokenStore, HashmapTwoFACodeStore, MockEmailClient, get_postgres_pool};
use tokio::sync::RwLock;
use auth_service::utils::constants::{prod, DATABASE_URL};

#[tokio::main]
async fn main() {

    let pg_pool = configure_postgresql().await;

    let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let banner_token_store =
        Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_code_store =
        Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client =
        Arc::new(MockEmailClient);

    let app_state = AppState::new(user_store, banner_token_store, two_fa_code_store,
                                  email_client);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}
