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
const ORG_PROMPT: &'static str = "Select the organization you want to clone repos from... ARROW KEYS: Next/Prev Page SPACE: Select, ENTER: Confirm";
const REPO_PROMPT: &'static str = "Select the repos you want to clone... ARROW KEYS: Next/Prev Page SPACE: Select, ENTER: Confirm";
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
            .with_prompt("Select the organization you want to clone repos from... ARROW KEYS: Next/Prev Page SPACE: Select, ENTER: Confirm ")
            .items(&items)
            .interact()
            .unwrap();
        let mut vec = Vec::new();
        vec.push(selection);
        vec
    }
}

async fn fetch(url: &str, token: &str) -> Result<Value, Error> {
    let client: reqwest::Client = reqwest::Client::new();
    let res: reqwest::Response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "reqwest")
        .send()
        .await?;

    let body = res.bytes().await?; // Get response body as bytes
    let json_response: Value = serde_json::from_slice(&body).unwrap();
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

async fn fetch_orgs(token: &str) -> Result<Vec<Value>, Error> {
    let json_response = fetch("https://api.github.com/user/orgs", &token).await?;
    let array: Vec<Value> = json_response.as_array().unwrap().to_owned();
    Ok(array)
}

async fn get_repo_links(repo_link: &str, qparams: &str, token: &str) -> Result<Vec<Value>, Error> {
    let repo_link = repo_link.trim_matches('"').to_owned() + qparams;
    let json_response = fetch(&repo_link, &token).await?;
    let array: Vec<Value> = json_response.as_array().unwrap().to_owned();
    Ok(array)
}

fn clone_selected_repos(selected_repos: Vec<Value>, gh_token: &str) {
    let mut repo_urls: Vec<String> = Vec::new();
    selected_repos.iter().for_each(|repo: &Value| {
        let repo_url: String = repo["clone_url"].to_string();
        repo_urls.push(repo_url);
    });

    repo_urls.iter().for_each(|url| {
        clone_repo(url, gh_token);
    });
}
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Args = Args::parse();
    let body: Vec<Value>;
    if let Ok(orgs) = fetch_orgs(&args.gh_token).await {
        body = orgs;
    } else {
        println!("Unable to fetch organizations");
        std::process::exit(1);
    }
    let selection: Vec<usize> = retrieve_selection(
        false,
        &body
            .iter()
            .map(|org| org["login"].to_string())
            .collect::<Vec<String>>(),
        ORG_PROMPT,
    );

    let current_repo: String = body[selection[0]]["repos_url"].to_string();
    let repos: Vec<Value>;
    println!("{}", current_repo);
    if let Ok(repos_ok) = get_repo_links(&current_repo, "?per_page=100", &args.gh_token).await {
        repos = repos_ok //.to_owned();
    } else {
        println!("Invalid token");
        std::process::exit(1);
    };
    let selection: Vec<usize> = retrieve_selection(
        true,
        &repos
            .iter()
            .map(|repo| repo["name"].to_string())
            .collect::<Vec<String>>(),
        REPO_PROMPT,
    );

    let selected_raw_body = selection
        .iter()
        .map(|item: &usize| repos[*item].to_owned())
        .collect::<Vec<Value>>();
    clone_selected_repos(selected_raw_body, &args.gh_token);
    Ok(())
}

//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
