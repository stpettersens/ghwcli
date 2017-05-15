#[derive(Debug, Serialize, Deserialize)]
pub struct GitHub {
    username: String,
    password: String,
}

impl GitHub {
    pub fn new(username: &str, password: &str) -> GitHub {
        GitHub {
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }
    pub fn get_root_url(&self) -> String {
        "https://github.com".to_owned()
    }
    pub fn get_base_url(&self) -> String {
        "https://raw.githubusercontent.com".to_owned()
    }
    pub fn get_url_frag(&self) -> String {
        format!("{}/{}/", self.get_base_url(), self.username)
    }
    pub fn get_index_frag(&self) -> String {
        format!("{}/{}/", self.get_root_url(), self.username)
    }
}
