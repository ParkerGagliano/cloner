use std::intrinsics::r#try;
use std::os::unix::process;

use clap::Parser;
use dialoguer::{MultiSelect, Select};
use reqwest;
use reqwest::Error;
use serde_json::Value;
use tokio;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    gh_token: String,
}

async fn fetch(url: &str, token: &str) -> Result<Value, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "reqwest")
        .send()
        .await?;

    let body = res.bytes().await?; // Get response body as bytes
    let json_response: Value = serde_json::from_slice(&body).unwrap();
    Ok(json_response)
}

async fn fetch_orgs(token: &str) -> Result<Value, Error> {
    let json_response = fetch("https://api.github.com/user/orgs", &token).await?;
    Ok(json_response)
}

async fn fetch_repos(repo_link: &str, token: &str) -> Result<Value, Error> {
    let repo_link = repo_link.trim_matches('"');
    let json_response = fetch(&repo_link, &token).await?;
    Ok(json_response)
}

fn clone_repo(repo_url: &str, gh_token: &str) {
    let repo_url = &repo_url.trim_matches('"')[8..];
    println!("{}", repo_url);
    let clone_url = "https://".to_owned() + gh_token + "@" + &repo_url;

    let mut command = String::from("git clone ");
    command.push_str(&clone_url);

    println!("{}", command);
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn clone_selected_repos(selected_repos: Vec<Value>, gh_token: &str) {
    let mut repo_urls = Vec::new();
    selected_repos.iter().for_each(|repo| {
        let repo_url = repo["clone_url"].to_string();
        repo_urls.push(repo_url);
    });

    repo_urls.iter().for_each(|url| {
        clone_repo(url, gh_token);
    });
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();
    let body;
    if let Ok(orgs) = fetch_orgs(&args.gh_token).await {
        body = orgs;
    } else {
        println!("Invalid token");
        std::process::exit(1);
    }

    let body: &Vec<Value> = body.as_array().unwrap();

    let selection: usize = Select::new()
        .with_prompt("Select the organization you want to clone repos from... ARROW KEYS: Next/Prev Page SPACE: Select, ENTER: Confirm ")
        .items(&body.iter().map(|org| org["login"].to_string()).collect::<Vec<String>>())
        .interact()
        .unwrap();

    let current_repo: String = body[selection]["repos_url"].to_string();
    let repos: &Vec<Value>;
    if let Ok(repos_ok) = fetch_repos(&(current_repo + "?per_page=100"), &args.gh_token).await {
        if let Some(valid_repos) = repos_ok.as_array() {
            // as_array doesnt return error or not, it returns Some or None
            repos = valid_repos;
        } else {
            println!("No Repos Found");
            std::process::exit(1);
        }
    } else {
        println!("Invalid token");
        std::process::exit(1);
    };

    let selection = MultiSelect::new()
        .with_prompt("Select the repos you want to clone... ARROW KEYS: Next/Prev Page SPACE: Select, ENTER: Confirm ")
        .items(&repos
            .iter()
            .map(|repo| repo["name"].to_string())
            .collect::<Vec<String>>())
        .interact()
        .unwrap();

    let selected_raw_body = selection
        .iter()
        .map(|item: &usize| repos[*item].to_owned())
        .collect::<Vec<Value>>();
    clone_selected_repos(selected_raw_body, &args.gh_token);
    Ok(())
}

//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
