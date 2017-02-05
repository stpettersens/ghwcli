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
extern crate regex;
extern crate select;
extern crate clioptions;
use github::GitHub;
use project::Project;
use curl::easy::Easy;
use text_diff::{diff, print_diff, Difference};
use rustc_serialize::json;
use rustc_serialize::json::Json;
use regex::Regex;
use select::document::Document;
use select::predicate::Name;
use clioptions::CliOptions;
use std::io::{stdin, stdout, Read, Write};
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

fn split_url_from_blob(burl: &str) -> String {
    let split = burl.split("/blob/");
    let mut url: Vec<String> = Vec::new();
    for s in split {
        if s.to_owned().len() > 0 {
            url.push(s.to_owned())
        }
    }
    url.join("/")
}

fn split_dir_from_tree(url: &str) -> String {
    let split = url.split("/tree/");
    let mut dir: Vec<String> = Vec::new();
    for s in split {
        if s.to_owned().len() > 0 {
            dir.push(s.to_owned());
        }
    }
    dir.remove(0);
    dir.join("/")
}

fn retrieve_file(gh: &GitHub, project: &Project, file: &str, verbose: bool, index: u32) {
    let mut c = Easy::new();
    if index == 1 {
        c.url(&format!("{}{}", gh.get_index_frag(), project.get_index_frag())).unwrap();
        if verbose {
            println!("Retrieving index: {}{}", gh.get_index_frag(), project.get_index_frag());
        }
    } else if index == 2 {
        println!("Retrieving subindex: {}{}{}", gh.get_index_frag(), project.get_tree_frag(), file);
        c.url(&format!("{}{}{}", gh.get_index_frag(), project.get_tree_frag(), file)).unwrap();
    } else {
        c.url(&format!("{}{}", gh.get_base_url(), file)).unwrap();
    }
    let pw = "_git_";
    let mut out = format!("{}/{}", pw, file);
    if index == 2 {
        out = format!("{}/index.html", pw);
    }
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
    if verbose && index == 0 {
        println!("Retrieved file: {}{} [{}]", 
        gh.get_base_url(), file, c.response_code().unwrap());
    }
}

fn get_index(gh: &GitHub, project: &Project, verbose: bool, dindex: u32, file: &str) -> Vec<String> {
    retrieve_file(&gh, &project, file, verbose, dindex);
    let index = "_git_/index.html";
    let mut f = File::open(&index).unwrap();
    let mut html = String::new();
    let _ = f.read_to_string(&mut html);
    let mut links: Vec<String> = Vec::new();
    for node in Document::from_str(&html).find(Name("a")).iter() {
        links.push(node.attr("href").unwrap().to_owned());
    }
    let _ = fs::remove_file(&index); // !!!
    links
}


fn get_files(gh: &GitHub, project: &Project, verbose: bool, dindex: u32, file: &str) 
-> (Vec<String>, Vec<String>) {
    let links: Vec<String> = get_index(&gh, &project, verbose, dindex, &file);
    let mut files: Vec<String> = Vec::new();
    let mut branches: Vec<String> = Vec::new();
    for link in &links {
        let mut p = Regex::new("/blob/").unwrap();
        if p.is_match(&link) {
            files.push(split_url_from_blob(&link));
        }
        p = Regex::new("/tree/").unwrap();
        if p.is_match(&link) {
            branches.push(split_dir_from_tree(&link.clone()));
        }
    }
    //println!("Links: {:?}", files); // !!!
    if dindex < 2 {
        files.remove(0);
    }
    (files, branches)
}

fn retrieve_repo(gh: &GitHub, project: &Project, verbose: bool) {
    let (files, branches) = get_files(&gh, &project, verbose, 1, "index.html");
    for file in files {
        retrieve_file(&gh, &project, &file, verbose, 0);
    }
    for (i, branch) in branches.iter().enumerate() {
        if i == 2 {
            let (filess, branchess) = get_files(&gh, &project, verbose, 2, &branch);
            for file in filess {
                retrieve_file(&gh, &project, &file, verbose, 0);
            }
            //println!("{:?}", filess);
            //let index = get_index(&gh, &project, true, 2, &branch);
            //println!("{:?}", index);
        }
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

fn get_input(prompt: &str) -> String {
    println!("{}? ", prompt);
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(error) => {
            println!("Stdin Error: {}", error);
            exit(-1);
        }
    }
    input.trim().to_owned()
}

fn write_gh_configuration(conf: &str) {
    let username = get_input("Username");
    let password = get_input("Password");
    let gh = GitHub::new(&username, &password);
    let o = json::encode(&gh).unwrap();
    write_common_configuration(conf, &o);
}

fn write_project_configuration(conf: &str) {
    let name = get_input("Project name");
    let branch = get_input("Branch");
    let project = Project::new(&name, &branch);
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

fn display_version() {
    println!("ghwcli v. 0.1.0");
    println!("This program uses libcurl (https://curl.haxx.se)");
    exit(0);
}

fn display_error(program: &str, err: &str) {
    println!("Error: {}.\n", err);
    display_usage(program, -1);
}

fn display_usage(program: &str, code: i32) {
    println!("ghwcli (GitHub Web Command Line).");
    println!("Alternative command line utility to commit to GitHub.");
    println!("Copyright 2017 Sam Saint-Pettersen.");
    println!("\nReleased under the MIT License.");
    println!("\nUsage: {} <command> [<repo>][<options>]", program);
    println!("\nCommands:\n");
    println!("clone : Clone the configured project or at specified GitHub repo.");
    println!("diff : See the differences between working directory and GitHub repo.");
    println!("commit : Commit the local changes back to the GitHub repo.");
    println!("push : Push the local changes back to the GitHub repo.");
    println!("\nOptions:\n");
    println!("-h | --help : Display this usage information and exit.");
    println!("-v | --version : Display program version and exit.");
    println!("-q | --quiet : Do not output non-error messages to stdout.");
    exit(code);
}

fn main() {
    let cli = CliOptions::new("ghwcli");
    let program = cli.get_program();

    // ---------------------------------
    let ghconf = ".github.json";
    let prjconf = ".project.json";
    // ---------------------------------

    let mut gh: GitHub = GitHub::new("u", "p");
    let mut project: Project = Project::new("n", "b");
    let mut repo = String::new();
    let mut verbose = true;
    let mut op = -1;

    if cli.get_num() > 1 {
        for (i, a) in cli.get_args().iter().enumerate() {
            match a.trim() {
                "-h" | "--help" => display_usage(&program, 0),
                "-v" | "--version" => display_version(),
                "-q" | "--quiet" => verbose = false,
                "clone" => {
                    op = 0;
                    repo = cli.next_argument(i);
                },
                "configure" => op = 1,
                _ => continue,
            }
        }
    } else {
        display_error(&program, "No options provided");
    }
    
    if repo.is_empty() {
        if !Path::new(ghconf).exists() {
            write_gh_configuration(ghconf)
        }

        if !Path::new(prjconf).exists() {
            write_project_configuration(prjconf);
        }

        gh = load_gh_configuration(ghconf);
        project = load_project_configuration(prjconf);
    }
    match op {
        0 => {
            if !repo.is_empty() {
                let p = Regex::new(r"(\w+)/([\w-]+)").unwrap();
                for cap in p.captures_iter(&repo) {
                    gh = GitHub::new(&cap[1], "-");
                    project = Project::new(&cap[2], "master");
                }
            }
            retrieve_repo(&gh, &project, verbose);
        },
        1 => {
            write_gh_configuration(ghconf);
            write_project_configuration(prjconf);
        },
        _ => {}
    }
}
