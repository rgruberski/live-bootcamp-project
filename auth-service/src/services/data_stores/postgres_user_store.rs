use std::error::Error;
// use std::future::Future;
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
// use sqlx::postgres::PgRow;
use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {

        let password_hash = match compute_password_hash(user.password.as_ref()) {
            Ok(password_hash) => password_hash,
            Err(_) => return Err(UserStoreError::UnexpectedError)
        };

        match sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            &user.email.to_string(),
            &password_hash,
            &user.requires_2fa
        )
            .execute(&self.pool)
            .await {
            Ok(_) => Ok(()),
            Err(_) => Err(UserStoreError::UnexpectedError)
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {

        sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email.as_ref())
            .map(|row: PgRow| Ok(User {
                email: Email::parse(row.get("email")).unwrap(),
                password: Password::parse(row.get("password")).unwrap(),
                requires_2fa: row.get("requires_2fa"),
            }))
            .fetch_optional(&self.pool)
            .await
            .unwrap().ok_or(UserStoreError::UserNotFound)?
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;

        verify_password_hash(
            user.password.as_ref(),
            password.as_ref(),
        )
        .map_err(|_| UserStoreError::InvalidCredentials)
    }
}

// Helper function to verify if a given password matches an expected hash
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let expected_password_hash: PasswordHash<'_> = PasswordHash::new(expected_password_hash)?;

    Argon2::default()
        .verify_password(password_candidate.as_bytes(), &expected_password_hash)
        .map_err(|e| e.into())
}

// Helper function to hash passwords before persisting them in the database.
// TODO: Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error>> {
    let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None)?,
    )
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}
