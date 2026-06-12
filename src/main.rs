use anyhow::{Context, Result};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};

/// Extract comprehensive GitHub repository details as JSON.
#[derive(Parser, Debug)]
#[command(name = "gh-repo-dump", version, about, long_about = None)]
struct Cli {
    /// GitHub repository in owner/repo format
    #[arg(value_name = "OWNER/REPO")]
    repository: String,

    /// GitHub personal access token (also via GITHUB_TOKEN env var)
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,

    /// Output JSON file path (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Per-endpoint request timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Verbose stderr logging
    #[arg(short, long)]
    verbose: bool,
}

const API_BASE: &str = "https://api.github.com";

#[derive(Clone)]
struct GhClient {
    client: reqwest::Client,
    base: String,
}

impl GhClient {
    fn new(token: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gh-repo-dump/0.1.0"));
        if let Some(t) = token {
            let mut auth_val =
                HeaderValue::from_str(&format!("Bearer {}", t)).context("invalid token")?;
            auth_val.set_sensitive(true);
            headers.insert("Authorization", auth_val);
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(GhClient {
            client,
            base: API_BASE.to_string(),
        })
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("{} {}: {}", status.as_u16(), url, body);
        }
        let json: Value = resp.json().await?;
        Ok(json)
    }

    async fn get_paginated(&self, path: &str, per_page: usize) -> Result<Vec<Value>> {
        let mut all: Vec<Value> = vec![];
        // Fetch up to 3 pages
        for page in 1..=3 {
            let paged_path = if path.contains('?') {
                format!("{}&per_page={}&page={}", path, per_page, page)
            } else {
                format!("{}?per_page={}&page={}", path, per_page, page)
            };
            let arr = self.get(&paged_path).await?;
            if let Some(arr) = arr.as_array() {
                let len = arr.len();
                all.extend(arr.iter().cloned());
                if len < per_page {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(all)
    }
}

struct RepoInfo {
    owner: String,
    repo: String,
}

impl RepoInfo {
    fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            anyhow::bail!("expected OWNER/REPO format, got: {}", input);
        }
        Ok(Self {
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
        })
    }
}

macro_rules! vlog {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose {
            eprintln!($($arg)*);
        }
    };
}

async fn fetch_all(client: &GhClient, owner: &str, repo: &str, verbose: bool) -> Result<Value> {
    let repo_path = format!("/repos/{}/{}", owner, repo);

    // Define endpoints as (label, path, paginated)
    let endpoints: Vec<(&str, String, bool)> = vec![
        ("repo", repo_path.clone(), false),
        ("releases", format!("{}/releases", repo_path), true),
        ("tags", format!("{}/tags", repo_path), true),
        ("languages", format!("{}/languages", repo_path), false),
        ("contributors", format!("{}/contributors", repo_path), true),
        ("branches", format!("{}/branches", repo_path), true),
        ("commits", format!("{}/commits", repo_path), true),
        ("readme", format!("{}/readme", repo_path), false),
        ("license", format!("{}/license", repo_path), false),
    ];

    let mut tasks = Vec::new();
    for (label, path, paginated) in &endpoints {
        let client_clone = client.clone();
        let label = *label;
        let path = path.clone();
        let paginated = *paginated;
        tasks.push(tokio::spawn(async move {
            let result = if paginated {
                client_clone
                    .get_paginated(&path, 30)
                    .await
                    .map(|v| json!(v))
            } else {
                client_clone.get(&path).await
            };
            (label, result)
        }));
    }

    let mut output = json!({});
    for task in tasks {
        let (label, result) = task.await?;
        match result {
            Ok(data) => {
                vlog!(verbose, "✓ {}", label);
                output[label] = data;
            }
            Err(e) => {
                vlog!(verbose, "✗ {}: {}", label, e);
                output[label] = json!({ "error": format!("{}", e) });
            }
        }
    }

    Ok(output)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let info = RepoInfo::parse(&cli.repository)?;

    if cli.verbose {
        eprintln!("Fetching details for {}/{}...", info.owner, info.repo);
    }

    let client = GhClient::new(cli.token.as_deref())?;
    let result = timeout(
        Duration::from_secs(cli.timeout.saturating_mul(3)),
        fetch_all(&client, &info.owner, &info.repo, cli.verbose),
    )
    .await
    .context("overall request timed out")??;

    let output_str = serde_json::to_string_pretty(&result)?;

    match cli.output {
        Some(path) => {
            std::fs::write(&path, &output_str)
                .context(format!("failed to write output to {}", path))?;
            eprintln!("Written to {}", path);
        }
        None => {
            println!("{}", output_str);
        }
    }

    Ok(())
}
