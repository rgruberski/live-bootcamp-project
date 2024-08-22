use super::UserStoreError;

use validator::validate_email;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(email: &str) -> Result<Self, UserStoreError> {
        match validate_email(email) {
            true => Ok(Email(email.to_string())),
            false => Err(UserStoreError::InvalidEmail),
        }
    }
}

impl AsRef<str> for Email {
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
            Email::parse("wrongemail.com").unwrap_err(),
            UserStoreError::InvalidEmail
        )
    }

    #[tokio::test]
    async fn test_parse_with_correct_value() {
        assert_eq!(
            Email::parse("mail@example.com").unwrap(),
            Email("mail@example.com".to_string())
        )
    }
}
