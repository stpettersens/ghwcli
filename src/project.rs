use std::fmt;

pub struct Project {
    name: String,
    branch: String,
}

impl Project {
    pub fn new(name: &str, branch: &str) -> Project {
        Project {
            name: name.to_owned(),
            branch: branch.to_owned(),
        }
    }
    pub fn get_url_frag(&self) -> &str {
        &format!("{}/{}", self.name, self.branch)
    }
}
