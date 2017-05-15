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
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate select;
extern crate clioptions;
use github::GitHub;
use project::Project;
use curl::easy::Easy as CurlRequest;
use text_diff::{diff, print_diff, Difference};
use regex::Regex;
use select::document::Document;
use select::predicate::Name;
use clioptions::CliOptions;
use std::io::{stdin, stdout, Read, Write};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::exit;

fn split_from_blob(url: &str) -> String {
    str::replace(url, "/blob/", "/")
}

fn split_dir_from_file(df: &str) -> String {
    let split = df.split("/");
    let mut path: Vec<String> = Vec::new();
    for s in split {
        if s.to_owned().len() > 0 {
            path.push(s.to_owned());
        }
    }
    path.pop();
    path.join("/")
}

fn split_file_from_url(url: &str, branch: &str) -> String {
    let split = url.split("/");
    let mut path: Vec<String> = Vec::new();
    for s in split {
        if s.to_owned().len() > 0 {
            path.push(s.to_owned());
        }
    }
    let idx = path.len() - 2;
    if path[idx] != branch {
        return format!("{}/{}", path[idx], path[idx + 1]);
    }
    path[idx + 1].to_owned()
}

fn split_double(url: &str, gh: &GitHub) -> String {
    let split = url.split("//");
    let mut path: Vec<String> = Vec::new();
    for s in split {
        if s.to_owned().len() > 0 {
            path.push(s.to_owned());
        }
    }
    format!("{}/{}", gh.get_root_url(), path[path.len() - 1])
}

fn retrieve_file(url: &str, branch: &str, html: bool, verbose: bool) {
    let mut c = CurlRequest::new();
    c.url(url).unwrap();
    let mut w = File::create("__repo__index.html").unwrap();
    if !html {
        let p = "__git__";
        if !Path::new(&p).exists() {
            let _ = fs::create_dir_all(p.clone());
        }
        let dir = format!("{}/{}", p, 
        split_dir_from_file(&split_file_from_url(&url, &branch)));
        if !Path::new(&dir).exists() {
            let _ = fs::create_dir_all(dir);
        }
        let out = format!("{}/{}", p, split_file_from_url(&url, &branch));
        w = File::create(&out).unwrap();
    }
    c.write_function(move |data| {
        Ok(w.write(data).unwrap())
    }).unwrap();
    c.perform().unwrap();
    let response = c.response_code().unwrap();
    if verbose {
        if !html {
            println!("Retrieved file [{}]: {}", response, url);
        }
    }
}

fn clean_up_index() {
    let idx = "__repo__index.html";
    if Path::new(idx).exists() {
        let _ = fs::remove_file(idx);
    }
}

fn get_links() -> Vec<String> {
    let mut f = File::open("__repo__index.html").unwrap();
    let mut html = String::new();
    let _ = f.read_to_string(&mut html);
    let mut links: Vec<String> = Vec::new();
    for node in Document::from_str(&html).find(Name("a")).iter() {
        links.push(node.attr("href").unwrap().to_owned());
    }
    links
}

fn get_links_from(url: &str, branch: &str, verbose: bool) -> Vec<String> {
    retrieve_file(url, branch, true, verbose);
    get_links()
}

fn tree_links_from(url: &str, gh: &GitHub, prj: &Project, verbose: bool) -> Vec<String> {
    let links = get_links_from(url, &prj.get_branch(), verbose);
    let mut tree: Vec<String> = Vec::new();
    for link in links {
        let mut p = Regex::new("https://").unwrap();
        if p.is_match(&link) {
            continue;
        }
        p = Regex::new(&format!("/tree/{}/", &prj.get_branch())).unwrap();
        if p.is_match(&link) {
            tree.push(format!("{}{}", gh.get_index_frag(), link));
        }
    }
    tree
}

fn blob_links_from(url: &str, gh: &GitHub, prj: &Project, verbose: bool) -> Vec<String> {
    let links = get_links_from(url, &prj.get_branch(), verbose);
    let mut blobs: Vec<String> = Vec::new();
    for link in links {
        let mut p = Regex::new("https://").unwrap();
        if p.is_match(&link) {
            continue;
        }
        p = Regex::new(&format!("/blob/{}/", &prj.get_branch())).unwrap();
        if p.is_match(&link) {
            blobs.push(format!("{}{}", gh.get_base_url(), split_from_blob(&link)));
        }
    }
    blobs
}

fn download_files(downloaded: Vec<String>, files: Vec<String>, 
prj: &Project, verbose: bool) -> Vec<String> {
    let mut dld: Vec<String> = Vec::new();
    for file in files {
        if !downloaded.contains(&file) {
            retrieve_file(&file, &prj.get_branch(), false, verbose);
            dld.push(file.to_owned());
        }
    }
    dld
}

fn retrieve_repo(gh: &GitHub, prj: &Project, verbose: bool) {
    let url = format!("{}{}", gh.get_index_frag(), prj.get_index_frag());
    let tree = tree_links_from(&url, &gh, &prj, verbose);
    let blobs = blob_links_from(&url, &gh, &prj, verbose);
    let mut adld: Vec<String> = Vec::new();
    let mut dld = download_files(adld.clone(), blobs, &prj, verbose);
    adld.append(&mut dld);
    for t in tree {
        let tree = tree_links_from(&split_double(&t, &gh), &gh, &prj, verbose);
        let blobs = blob_links_from(&split_double(&t, &gh), &gh, &prj, verbose);
        let mut dld = download_files(adld.clone(), blobs, &prj, verbose);
        adld.append(&mut dld);
        for t in tree {
            let tree = tree_links_from(&split_double(&t, &gh), &gh, &prj, verbose);
            let blobs = blob_links_from(&split_double(&t, &gh), &gh, &prj, verbose);
            let mut dld = download_files(adld.clone(), blobs, &prj, verbose);
            adld.append(&mut dld);
            for t in tree {
                let blobs = blob_links_from(&split_double(&t, &gh), &gh, &prj, verbose);
                let mut dld = download_files(adld.clone(), blobs, &prj, verbose);
                adld.append(&mut dld);
            }
        }
    }
    clean_up_index();
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
    let j = serde_json::to_string(&gh).unwrap();
    write_common_configuration(conf, &j);
}

fn write_project_configuration(conf: &str) {
    let name = get_input("Project name");
    let branch = get_input("Branch");
    let project = Project::new(&name, &branch);
    let j = serde_json::to_string(&project).unwrap();
    write_common_configuration(conf, &j);
}

fn load_common_configuration(conf: &str) -> String {
    let mut lines = String::new();
    let mut file = File::open(conf).unwrap();
    let _ = file.read_to_string(&mut lines);
    lines
}

fn load_gh_configuration(conf: &str) -> GitHub {
    //let ghj = Json::from_str(&load_common_configuration(&conf)).unwrap();
    //json::decode(&ghj.to_string()).unwrap()
    serde_json::from_str(&load_common_configuration(&conf)).unwrap()
}

fn load_project_configuration(conf: &str) -> Project {
    //let prj = Json::from_str(&load_common_configuration(&conf)).unwrap();
    //json::decode(&prj.to_string()).unwrap()
    serde_json::from_str(&load_common_configuration(&conf)).unwrap()
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
