use std::fmt;

pub struct GitHub {
    user: &str,
    password: &str,
}

impl GitHub {
    pub fn new(user: &str, password: &str) -> GitHub {
        GitHub {
            user: user,
            password: password,
        }
    }
    pub fn get_url_frag(&self) -> &str {
        &format!("https://raw.githubusercontent.com/{}/", self.user)
    }
}
