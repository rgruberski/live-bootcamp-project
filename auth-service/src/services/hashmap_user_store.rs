use std::collections::HashMap;

use crate::domain::{User, UserStore, UserStoreError};

// #[derive(Debug, PartialEq)]
// pub enum UserStoreError {
//     UserAlreadyExists,
//     UserNotFound,
//     InvalidCredentials,
//     UnexpectedError,
// }

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.clone(), user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if &user.password == password {
                    Ok(())
                }
                else {
                    Err(UserStoreError::InvalidCredentials)
                }
            },
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        
        let mut user_store = HashmapUserStore::default();

        let user = User::new(
            String::from("user@example.com"),
            String::from("password"),
            false
        );

        assert_eq!(user_store.add_user(user.clone()).await, Ok(()));

        assert_eq!(
            user_store.add_user(user).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        
        let mut user_store = HashmapUserStore::default();

        let user = User::new(
            String::from("user@example.com"),
            String::from("password"),
            false
        );

        assert_eq!(user_store.add_user(user.clone()).await, Ok(()));

        assert_eq!(
            user_store
                .get_user(&user.email)
                .await
                .expect("Failed to get the user"),
            user
        );

        assert_eq!(
            user_store.get_user("another@example.com").await,
            Err(UserStoreError::UserNotFound)
        );
    }

    #[tokio::test]
    async fn test_validate_user() {
        
        let mut user_store = HashmapUserStore::default();

        let user = User::new(
            String::from("user@example.com"),
            String::from("password"),
            false
        );

        assert_eq!(user_store.add_user(user.clone()).await, Ok(()));

        assert_eq!(
            user_store.validate_user(&user.email, &user.password).await,
            Ok(())
        );

        assert_eq!(
            user_store.validate_user(&user.email, "wrong password").await,
            Err(UserStoreError::InvalidCredentials)
        );
    }
}