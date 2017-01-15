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
use std::fs;
use std::fs::File;
use std::path::Path;

fn split_path_from_file(pathstr: &str) -> String {
    let split = pathstr.split("/");
    let mut path: Vec<String> = Vec::new();
    for s in split {
        if s.to_owned().len() > 0 {
            path.push(s.to_owned())
        }
    }
    path.pop();
    path.join("/")
}

fn retrieve_file(gh: GitHub, project: Project, file: &str, verbose: bool) {
    let mut c = Easy::new();
    c.url(&format!("{}{}{}", gh.get_url_frag(), project.get_url_frag(), file)).unwrap();
    let pw = "_git_";
    let out = format!("{}/{}", pw, file);
    let p = split_path_from_file(&out);
    println!("{}", p);
    if !Path::new(&p).exists() {
        let _ = fs::create_dir_all(p);
    }
    let mut w = File::create(out).unwrap();
    c.write_function(move |data| {
        Ok(w.write(data).unwrap())
    }).unwrap();
    c.perform().unwrap();
    println!("{:?}", c.response_code().unwrap());
    if verbose {
        println!("Retrieving file: {}{}{} [{}]", 
        gh.get_url_frag(), project.get_url_frag(), file, c.response_code().unwrap());
    }
}

fn main() {
    let gh = GitHub::new("stpettersens", "dummy123");
    let project = Project::new("touch", "master");
    let file = "src/dne.rs";
    retrieve_file(gh, project, &file, true);
}
