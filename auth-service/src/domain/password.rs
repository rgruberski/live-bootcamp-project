use super::UserStoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn parse(password: &str) -> Result<Self, UserStoreError> {
        match password.chars().count() >= 8 {
            true => Ok(Password(password.to_string())),
            false => Err(UserStoreError::InvalidCredentials),
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_with_incorrect_value() {
        assert_eq!(
            Password::parse("pass").unwrap_err(),
            UserStoreError::InvalidCredentials
        )
    }

    #[tokio::test]
    async fn test_parse_with_correct_value() {
        assert_eq!(
            Password::parse("password123").unwrap(),
            Password("password123".to_string())
        )
    }
}
