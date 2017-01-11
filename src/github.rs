use std::fmt;

pub struct GitHub {
    user: String,
    password: String,
}

impl GitHub {
    pub fn new(user: &str, password: &str) -> GitHub {
        GitHub {
            user: user.to_owned(),
            password: password.to_owned(),
        }
    }
    pub fn get_url_frag(&self) -> &str {
        &format!("https://raw.githubusercontent.com/{}/", &self.user)
    }
}
