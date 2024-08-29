use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

// TODO: implement TwoFACodeStore for HashmapTwoFACodeStore

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(&mut self, email: Email, login_attempt_id: LoginAttemptId, code: TwoFACode)
        -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email)
        -> Result<(), TwoFACodeStoreError> {
        self.codes
            .remove(email)
            .map(|_| ())
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }

    async fn get_code(&self, email: &Email)
        -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes
            .get(email)
            .map(|user| user.clone())
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_add_code() {

        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
        let fa_code = TwoFACode::parse("123456".to_string()).unwrap();

        assert!(store
            .add_code(email.clone(), login_attempt_id.clone(), fa_code.clone())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_remove_code() {

        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
        let fa_code = TwoFACode::parse("123456".to_string()).unwrap();

        assert!(store
            .add_code(email.clone(), login_attempt_id.clone(), fa_code.clone())
            .await
            .is_ok());

        let code_store = store.get_code(&email).await.unwrap();

        assert_eq!(code_store, (login_attempt_id, fa_code));

        store.remove_code(&email).await.unwrap();

        assert!(store.get_code(&email).await.is_err());
    }

    #[tokio::test]
    async fn test_get_code() {

        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("user@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::parse(Uuid::new_v4().to_string()).unwrap();
        let fa_code = TwoFACode::parse("123456".to_string()).unwrap();

        assert!(store
            .add_code(email.clone(), login_attempt_id.clone(), fa_code.clone())
            .await
            .is_ok());

        let code_store = store.get_code(&email).await.unwrap();

        assert_eq!(code_store, (login_attempt_id, fa_code));
    }
}
