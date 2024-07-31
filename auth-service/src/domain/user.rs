#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> User {
        Self {
            email: email,
            password: password,
            requires_2fa: requires_2fa
        }
    }
}
