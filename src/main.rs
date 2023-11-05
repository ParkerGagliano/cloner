use std::clone;

use clap::Parser;

use dialoguer::MultiSelect;
use reqwest::Error;
use reqwest::{self, header::AUTHORIZATION};
use serde_json::Value;
use tokio;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    gh_token: String,

    #[arg(short, long)]
    username: String,
}

async fn fetch_orgs(token: &str) -> Result<Value, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/user/orgs")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "reqwest")
        .send()
        .await?;

    let body = res.bytes().await?; // Get response body as bytes
    let json_response: Value = serde_json::from_slice(&body).unwrap();
    Ok(json_response)
}

async fn fetch_repos(repo_link: &str, token: &str) -> Result<Value, Error> {
    let client = reqwest::Client::new();
    let repo_link = repo_link.trim_matches('"');
    println!("{}", repo_link);
    let res = client
        .get(repo_link.to_owned() + "?per_page=100")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "reqwest")
        .send()
        .await?;
    let body = res.bytes().await?;
    let json_response: Value = serde_json::from_slice(&body).unwrap();
    Ok(json_response)
}

fn handle_repo_options(fetch_repos_body: Value) -> Result<Vec<Value>, Error> {
    let body = fetch_repos_body.as_array().unwrap();
    body.iter().for_each(|repo| println!("{}", repo["name"]));

    Ok(body.to_owned())
}

fn clone_repo(repo_url: &str, username: &str, gh_token: &str) {
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

fn handle_selected_repos(selected_repos: Vec<Value>, username: &str, gh_token: &str) {
    let mut repo_urls = Vec::new();
    selected_repos.iter().for_each(|repo| {
        let repo_url = repo["clone_url"].to_string();
        repo_urls.push(repo_url);
    });

    repo_urls.iter().for_each(|url| {
        clone_repo(url, username, gh_token);
    });
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();
    let body = fetch_orgs(&args.gh_token).await?;

    let body = body.as_array().unwrap();

    body.iter().for_each(|org| println!("{}", org["login"]));

    //get user input
    let mut input = String::new();
    println!("Enter the name of the organization you want to clone");
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().parse::<usize>().unwrap();

    let current_repo: String = body[input]["repos_url"].to_string();
    println!("The current repo is {}", current_repo);

    let repos = fetch_repos(&current_repo, &args.gh_token).await;

    let raw_body = handle_repo_options(repos.unwrap()).unwrap();

    let items = raw_body
        .iter()
        .map(|repo| repo["name"].to_string())
        .collect::<Vec<String>>();

    //println!("{:#?}", body);

    //let items = vec!["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];
    let selection = MultiSelect::new()
        .with_prompt("Select the repos you want to clone... ARROW KEYS: Next/Prev Page SPACE: Select, ENTER: Confirm ")
        .items(&items)
        .interact()
        .unwrap();

    let selected_raw_body = selection
        .iter()
        .map(|item| raw_body[*item].to_owned())
        .collect::<Vec<Value>>();

    println!("You selected these repos: {:?}", selected_raw_body);

    handle_selected_repos(selected_raw_body, &args.username, &args.gh_token);
    Ok(())
}

//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
