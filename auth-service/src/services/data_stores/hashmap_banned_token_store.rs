use std::collections::HashSet;
use secrecy::{ExposeSecret, Secret};
use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: HashSet<String>
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {

    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        /*match self.tokens.get(&token) {
            Some(e) => Err(BannedTokenStoreError::UnexpectedError),
            None => {
                self.tokens.insert(token);
                Ok(())
            }
        }*/

        self.tokens.insert(token.expose_secret().to_owned());
        Ok(())
    }

    async fn contains_token(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token.expose_secret()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {

        let mut token_store = HashsetBannedTokenStore::default();

        let token = Secret::new(String::from("token"));

        let result = token_store.add_token(token.clone()).await;

        assert!(result.is_ok());
        assert!(token_store.tokens.contains(token.expose_secret()));
    }

    // #[tokio::test]
    // async fn test_validate_token() {
    //
    //     let mut token_store = HashsetBannedTokenStore::default();
    //
    //     assert_eq!(token_store.add_token("token".to_string()).await, Ok(()));
    //
    //     assert_eq!(
    //         token_store.contains_token("valid_token").await,
    //         Ok(())
    //     );
    //
    //     assert_eq!(
    //         token_store.contains_token("token".to_string()).await,
    //         Err(BannedTokenStoreError::BannedToken)
    //     );
    // }
}