#[derive(Debug, RustcDecodable, RustcEncodable)]
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
    pub fn get_url_frag(&self) -> String {
        format!("{}/{}/", self.name, self.branch)
    }
    pub fn get_tree_frag(&self) -> String {
        format!("{}/tree/{}", self.name, self.branch)
    }
    pub fn get_index_frag(&self) -> String {
        format!("{}", self.name)
    }
}
