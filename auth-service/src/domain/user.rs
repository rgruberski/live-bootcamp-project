use super::{email::Email, password::Password};

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> User {
        Self {
            email: email,
            password: password,
            requires_2fa: requires_2fa
        }
    }
}
