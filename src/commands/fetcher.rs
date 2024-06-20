use reqwest;
use serde::Deserialize;
use std::process::Command;
use std::io::{self, Write};
use std::env;
use std::path::Path;
use tokio::time::{timeout, Duration};
use thiserror::Error;
// This program will run an asyncrounous Fetch Request to IPFS to load from a git commit hash from the rust-sgx
#[derive(Deserialize, Debug)]
struct RepoInfo {
    repo_url: String,
    commit_hash: String,
}

#[derive(Error, Debug)]
enum FetchError {
    #[error("Request timed out")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Request error")]
    Reqwest(#[from] reqwest::Error),
    #[error("HTTP error with status code {0}")]
    Http(reqwest::StatusCode),
}

async fn fetch_from_gateway(cid: &str, gateway: &str) -> Result<RepoInfo, FetchError> {
    let url = format!("{}/ipfs/{}", gateway, cid);
    println!("Fetching from URL: {}", url);

    // Set a timeout for the request
    let client = reqwest::Client::new();
    let response = timeout(Duration::from_secs(30), client.get(&url).send()).await;

    match response {
        Ok(Ok(resp)) => {
            if resp.status().is_success() {
                let repo_info = resp.json::<RepoInfo>().await?;
                Ok(repo_info)
            } else {
                Err(FetchError::Http(resp.status()))
            }
        }
        Ok(Err(e)) => Err(FetchError::Reqwest(e)),
        Err(e) => Err(FetchError::Timeout(e)),
    }
}

fn run_git_command(args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to execute git command");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}

pub fn run() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("Running defe-fetcher");

        // Prompt the user for the IPFS CID
        print!("Enter the IPFS CID: ");
        io::stdout().flush().unwrap();

        let mut cid = String::new();
        io::stdin().read_line(&mut cid).unwrap();
        let cid = cid.trim();

        // List of gateways to try
        let gateways = [
            "https://ipfs.io",
            "https://dweb.link",
            "https://cloudflare-ipfs.com",
        ];

        // Attempt to fetch the file from multiple gateways
        let repo_info = {
            let mut repo_info = None;
            for gateway in &gateways {
                match fetch_from_gateway(cid, gateway).await {
                    Ok(info) => {
                        repo_info = Some(info);
                        break;
                    }
                    Err(err) => {
                        eprintln!("Error fetching from {}: {}", gateway, err);
                    }
                }
            }
            repo_info
        };

        let repo_info = match repo_info {
            Some(info) => info,
            None => {
                eprintln!("Failed to fetch file from all gateways.");
                return;
            }
        };

        println!("Fetched repository info from IPFS: {:?}", repo_info);

        // Extract the repository name from the URL
        let repo_name = repo_info.repo_url
            .split('/')
            .last()
            .unwrap()
            .trim_end_matches(".git");

        // Check if the repository directory exists
        let repo_path = Path::new(repo_name);

        if repo_path.exists() {
            // Change directory to the cloned repository
            env::set_current_dir(repo_path).unwrap();
            println!("Changed directory to {}", repo_name);

            // Fetch only the specific commit to ensure the commit is accessible
            println!("Fetching the specific commit...");
            if let Err(err) = run_git_command(&["fetch", "origin", &repo_info.commit_hash]) {
                eprintln!("Git fetch failed: {}", err);
                return;
            }

            // Reset to the specific commit to replenish missing files
            println!("Resetting to the specific commit...");
            if let Err(err) = run_git_command(&["reset", "--hard", &repo_info.commit_hash]) {
                eprintln!("Git reset failed: {}", err);
                return;
            }

        } else {
            // Clone the GitHub repository with depth 1 to save space
            println!("Cloning the repository...");
            if let Err(err) = run_git_command(&["clone", "--depth", "1", "--branch", "main", &repo_info.repo_url]) {
                eprintln!("Git clone failed: {}", err);
                return;
            }

            // Change directory to the cloned repository
            env::set_current_dir(repo_name).unwrap();
            println!("Changed directory to {}", repo_name);

            // Fetch only the specific commit
            println!("Fetching the specific commit...");
            if let Err(err) = run_git_command(&["fetch", "origin", &repo_info.commit_hash]) {
                eprintln!("Git fetch failed: {}", err);
                return;
            }

            // Reset to the specific commit
            println!("Resetting to the specific commit...");
            if let Err(err) = run_git_command(&["reset", "--hard", &repo_info.commit_hash]) {
                eprintln!("Git reset failed: {}", err);
                return;
            }
        }

        println!("Checked out commit: {}", repo_info.commit_hash);
    });
}
