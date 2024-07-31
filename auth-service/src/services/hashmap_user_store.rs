use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        match self.users.get(&user.email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(user.email.clone(), user);
                Ok(())
            }
        }
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
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

        assert_eq!(user_store.add_user(user.clone()), Ok(()));

        assert_eq!(
            user_store.add_user(user),
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

        assert_eq!(user_store.add_user(user.clone()), Ok(()));

        assert_eq!(
            user_store
                .get_user(&user.email)
                .expect("Failed to get the user"),
            user
        );

        assert_eq!(
            user_store.get_user("another@example.com"),
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

        assert_eq!(user_store.add_user(user.clone()), Ok(()));

        assert_eq!(
            user_store.validate_user(&user.email, &user.password),
            Ok(())
        );

        assert_eq!(
            user_store.validate_user(&user.email, "wrong password"),
            Err(UserStoreError::InvalidCredentials)
        );
    }
}