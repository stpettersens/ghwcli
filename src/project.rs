use std::fmt;

pub struct Project {
    name: &str,
    branch: &str,
}

impl Project {
    pub fn new(name: &str, branch: &str) -> Project {
        Project {
            name: name,
            branch: branch,
        }
    }
    pub fn get_url_frag(&self) -> &str {
        &format!("{}/{}", self.name, self.branch)
    }
}
