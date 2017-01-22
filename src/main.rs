/*
    ghwcli (GitHub Web Command Line).
    Alternative command line utility to commit to GitHub.
    Copyright 2017 Sam Saint-Pettersen.

    Released under the MIT License.
*/

mod github;
mod project;
extern crate curl;
extern crate text_diff;
extern crate rustc_serialize;
extern crate clioptions;
use github::GitHub;
use project::Project;
use curl::easy::Easy;
use text_diff::{diff, print_diff, Difference};
use rustc_serialize::json;
use rustc_serialize::json::Json;
use clioptions::CliOptions;
use std::io::{Read, Write};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::exit;

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
    if !Path::new(&p).exists() {
        let _ = fs::create_dir_all(p);
    }
    let mut w = File::create(&out).unwrap();
    c.write_function(move |data| {
        Ok(w.write(data).unwrap())
    }).unwrap();
    c.perform().unwrap();
    if c.response_code().unwrap() != 200 {
        let _ = fs::remove_file(&out);
    }
    if verbose {
        println!("Retrieved file: {}{}{} [{}]", 
        gh.get_url_frag(), project.get_url_frag(), file, c.response_code().unwrap());
    }
}

fn check_for_diff(orig: &str, edit: &str) {
    let (dist, changeset) = diff(orig, edit, "");
    println!("dist: {:?}", dist);
    println!("cs: {:?}", changeset);
}

fn write_common_configuration(conf: &str, o: &str) {
    let mut w = File::create(conf).unwrap();
    let fo = format!("{}\n", o);
    let _ = w.write_all(fo.as_bytes());
}

fn write_gh_configuration(conf: &str) {
    let gh = GitHub::new("stpettersens", "-");
    let o = json::encode(&gh).unwrap();
    write_common_configuration(conf, &o);
}

fn write_project_configuration(conf: &str) {
    let project = Project::new("touch", "master");
    let o = json::encode(&project).unwrap();
    write_common_configuration(conf, &o);
}

fn load_common_configuration(conf: &str) -> String {
    let mut lines = String::new();
    let mut file = File::open(conf).unwrap();
    let _ = file.read_to_string(&mut lines);
    lines
}

fn load_gh_configuration(conf: &str) -> GitHub {
    let ghj = Json::from_str(&load_common_configuration(&conf)).unwrap();
    json::decode(&ghj.to_string()).unwrap()
}

fn load_project_configuration(conf: &str) -> Project {
    let prj = Json::from_str(&load_common_configuration(&conf)).unwrap();
    json::decode(&prj.to_string()).unwrap()
}

fn display_error(program: &str, err: &str) {
    println!("Error: {}.\n", err);
    display_usage(program, -1);
}

fn display_usage(program: &str, code: i32) {
    println!("Usage: {} <command> [<file>]", program);
    exit(code);
}

fn main() {
    let cli = CliOptions::new("ghwcli");
    let program = cli.get_program();

    // ---------------------------------
    let ghconf = ".github.json";
    let prjconf = ".project.json";
    // ---------------------------------

    if !Path::new(ghconf).exists() {
        write_gh_configuration(ghconf)
    }

    if !Path::new(prjconf).exists() {
        write_project_configuration(prjconf);
    }

    let gh: GitHub = load_gh_configuration(ghconf);
    let project: Project = load_project_configuration(prjconf);

    let mut file = String::new();
    let mut verbose = true;
    let mut op = -1;

    if cli.get_num() > 1 {
        for (i, a) in cli.get_args().iter().enumerate() {
            match a.trim() {
                "-h" | "--help" => display_usage(&program, 0),
                "clone" => {
                    op = 0;
                    file = cli.next_argument(i);
                },
                "configure" => op = 1,
                _ => continue,
            }
        }
    } else {
        display_error(&program, "No options provided");
    }
    match op {
        0 => retrieve_file(gh, project, &file, verbose),
        1 => {
            write_gh_configuration(ghconf);
            write_project_configuration(prjconf);
        },
        _ => {}
    }
}
