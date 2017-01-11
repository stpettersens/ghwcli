/*
    ghwcli (GitHub Web Command Line).
    Alternative command line utility to commit to GitHub.
    Copyright 2017 Sam Saint-Pettersen.

    Released under the MIT License.
*/

mod github;
mod project;
extern crate curl;
extern crate clioptions;
use github::GitHub;
use project::Project;
use curl::easy::Easy;
use clioptions::CliOptions;
use std::io::{stdout, Write};

fn retrieve_file(gh: GitHub, project: Project, file: &str) {
    println!("Retrieving file: {}{}{}", gh.get_url_frag(), project.get_url_frag(), file);
    let mut c = Easy::new();
    c.url(&format!("{}{}{}", gh.get_url_frag(), project.get_url_frag(), file)).unwrap();
    c.write_function(|data| {
        Ok(stdout().write(data).unwrap())
    }).unwrap();
    c.perform().unwrap();

    println!("{}", c.response_code().unwrap());
}

fn main() {
    let gh = GitHub::new("stpettersens", "dummy123");
    let project = Project::new("touch", "master");
    let file = "README.md";
    retrieve_file(gh, project, &file);
}
