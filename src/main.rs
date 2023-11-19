use clap::Parser;
use dialoguer::{MultiSelect, Select};
use reqwest::Error;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    str::from_utf8,
    thread::{self, JoinHandle},
};

use reqwest;
use tokio;

mod loader;

const ORG_PROMPT: &str = "Select the organization you want to clone repos from...\n\
    ← →: Next/Prev Page\n\
    ENTER: Confirm";

const REPO_PROMPT: &str = "Select the repos you want to clone...\n\
    ← →: Prev/Next Page\n\
    SPACE: Select\n\
    ENTER: Confirm";

const GITHUB_ORG_URL: &str = "https://api.github.com/user/orgs";
const PER_PAGE_PARAM: &str = "?per_page=100";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    gh_token: Option<String>,

    #[arg(short, long, default_value = "false")]
    update: bool,
}
#[derive(Debug, Deserialize)]
struct OrgInfo {
    repos_url: String,
    login: String,
}
#[derive(Deserialize, Debug, Clone)]
struct RepoInfo {
    clone_url: String,
    name: String,
}

fn retrieve_selection(multi: bool, items: &Vec<String>, prompt: &str) -> Vec<usize> {
    if multi {
        let selection: Vec<usize> = MultiSelect::new()
            .with_prompt(prompt)
            .items(&items)
            .interact()
            .unwrap();
        selection
    } else {
        let selection: usize = Select::new()
            .with_prompt(prompt)
            .items(&items)
            .interact()
            .unwrap();
        let mut vec = Vec::new();
        vec.push(selection);
        vec
    }
}

async fn fetch<T: DeserializeOwned>(url: &str, token: &str, qparams: &str) -> Result<T, Error> {
    let loader = loader::Loader::new();
    loader.start();
    let client: reqwest::Client = reqwest::Client::new();
    let res: reqwest::Response = client
        .get(url.to_owned() + qparams)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "reqwest")
        .send()
        .await?;
    if !res.status().is_success() {
        eprintln!("Error fetching: {}", res.status());
        std::process::exit(1); 
    }
    let body = res.bytes().await?;
    let json_response: T = serde_json::from_slice(&body).unwrap();
    loader.stop();
    Ok(json_response)
}

fn clone_repo(repo_url: &str, gh_token: &str) {
    let mut command = String::from("git clone ");
    let repo_url: &str = &repo_url.trim_matches('"')[8..];
    println!("Cloning Repo: {}", repo_url);
    command = command + "https://" + gh_token + "@" + &repo_url;

    let _output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
}

fn clone_selected_repos(selected_repos: Vec<RepoInfo>, gh_token: &str) {
    let mut repo_urls: Vec<String> = Vec::new();
    selected_repos.iter().for_each(|repo: &RepoInfo| {
        let repo_url: String = repo.clone_url.to_string();
        repo_urls.push(repo_url);
    });
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    repo_urls.iter().for_each(|url| {
        let url = url.to_owned();
        let gh_token = gh_token.to_owned();
        let current_thread = thread::spawn(move || {
            clone_repo(&url, &gh_token);
        });
        handles.push(current_thread);
    });

    handles
        .into_iter()
        .for_each(|handle: JoinHandle<()>| match handle.join() {
            Ok(_) => println!("Success"),
            Err(_) => println!("Thread failed"),
        });
}

fn token_handler(gh_token: &str) -> Result<String, ()> {
    if gh_token != "" {
        if !std::path::Path::new("token.txt").exists() {
            println!("Writing token to file...");
        } else {
            println!("Overwriting token file...");
        }
        std::fs::write("token.txt", gh_token).expect("Unable to write file");
        return Ok(gh_token.to_owned());
    } else {
        if let Ok(value) = std::fs::read_to_string("token.txt") {
            Ok(value)
        } else {
            Err(())
        }
    }
}

fn ls_directory() -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    let paths = std::fs::read_dir("./").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();
        files.push(path.to_string());
    }
    files
}

fn pull_all_repos() {
    let all_files: Vec<String> = ls_directory();
    let mut handles: Vec<JoinHandle<String>> = Vec::new();
    for file in all_files
        .iter()
        .filter(|&x| x != &"./token.txt" && x != &"./cloner")
    {
        let file = file.to_owned();
        let current_thread = thread::spawn(move || {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("cd {} && git pull", file))
                .output()
                .expect("idk fix this");
            if output.stderr.len() > 0 {
                return format!(
                    "Error Pulling: '{}': Shell Output: {}",
                    file,
                    from_utf8(&output.stderr).unwrap()
                );
            } else {
                return format!(
                    "Success Pulling '{}': Shell Output: {}",
                    file,
                    from_utf8(&output.stdout).unwrap()
                );
            }
        });
        handles.push(current_thread);
    }

    handles
        .into_iter()
        .for_each(|handle: JoinHandle<String>| match handle.join() {
            Ok(value) => println!("{}", value), //this will "always return an ok value even if the shell errors"
            Err(_) => println!("Thread failed"),
        });
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Args = Args::parse();
    if args.update {
        pull_all_repos();
        std::process::exit(0);
    }

    let gh_token = match token_handler(&args.gh_token.unwrap_or("".to_string())) {
        Ok(token) => token,
        Err(_) => {
            println!("No Token Provided or Found");
            std::process::exit(1);
        }
    };

    let org_choices: Vec<OrgInfo> = match fetch(GITHUB_ORG_URL, &gh_token, PER_PAGE_PARAM).await {
        Ok(orgs) => orgs,
        Err(err) => {
            eprintln!("Unable to fetch organizations: {}", err);
            std::process::exit(1);
        }
    };

    let org_selection_indexes: Vec<usize> = retrieve_selection(
        false,
        &org_choices
            .iter()
            .map(|org| org.login.to_string())
            .collect::<Vec<String>>(),
        ORG_PROMPT,
    );

    let org_repos_url: String = org_choices[org_selection_indexes[0]].repos_url.to_string();
    let repo_choices: Vec<RepoInfo> =
        match fetch(&org_repos_url.to_owned(), &gh_token, "?per_page=100").await {
            Ok(repos) => repos,
            Err(err) => {
                eprintln!("Unable to fetch repos: {}", err);
                std::process::exit(1);
            }
        };

    let selection: Vec<usize> = retrieve_selection(
        true,
        &repo_choices
            .iter()
            .map(|repo| repo.name.to_string())
            .collect::<Vec<String>>(),
        REPO_PROMPT,
    );

    let selected_repos = selection
        .iter()
        .map(|item: &usize| repo_choices[*item].to_owned())
        .collect::<Vec<RepoInfo>>();

    clone_selected_repos(selected_repos, &gh_token);

    Ok(())
}
