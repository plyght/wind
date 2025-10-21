use crate::{models::*, provider::CollabProvider};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

pub struct GitHubProvider {
    owner: String,
    repo: String,
    use_cli: bool,
    token: Option<String>,
}

impl GitHubProvider {
    pub async fn new(owner: String, repo: String) -> Result<Self> {
        let use_cli = which::which("gh").is_ok();
        let token = std::env::var("GH_TOKEN").ok();
        
        if !use_cli && token.is_none() {
            return Err(anyhow!(
                "GitHub integration requires either 'gh' CLI installed or GH_TOKEN environment variable set"
            ));
        }
        
        Ok(Self {
            owner,
            repo,
            use_cli,
            token,
        })
    }
    
    async fn gh_cli(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("gh")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn gh CLI")?
            .wait_with_output()
            .await
            .context("Failed to wait for gh CLI")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh CLI failed: {}", stderr));
        }
        
        Ok(String::from_utf8(output.stdout).context("Invalid UTF-8 from gh CLI")?)
    }
    
    async fn api_call(&self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value> {
        let token = self.token.as_ref()
            .ok_or_else(|| anyhow!("GH_TOKEN not set for API fallback"))?;
        
        let url = format!("https://api.github.com{}", endpoint);
        let client = reqwest::Client::new();
        let mut req = client.request(
            method.parse().context("Invalid HTTP method")?,
            &url,
        )
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "wind-collab")
        .header("Accept", "application/vnd.github.v3+json");
        
        if let Some(body) = body {
            req = req.json(&body);
        }
        
        let resp = req.send().await.context("API request failed")?;
        let status = resp.status();
        let text = resp.text().await?;
        
        if !status.is_success() {
            return Err(anyhow!("GitHub API error {}: {}", status, text));
        }
        
        serde_json::from_str(&text).context("Failed to parse API response")
    }
    
    fn prepare_body(&self, body: &str, stack_metadata: Option<&StackMetadata>) -> String {
        if let Some(metadata) = stack_metadata {
            format!("{}\n\n{}", body, metadata.serialize_for_body())
        } else {
            body.to_string()
        }
    }
}

#[async_trait]
impl CollabProvider for GitHubProvider {
    async fn create_pr(&self, req: CreatePrRequest) -> Result<PrRef> {
        let body = self.prepare_body(&req.body, req.stack_metadata.as_ref());
        
        if self.use_cli {
            let mut args = vec![
                "pr", "create",
                "--title", &req.title,
                "--body", &body,
                "--head", &req.head,
                "--base", &req.base,
            ];
            
            if req.draft {
                args.push("--draft");
            }
            
            let output = self.gh_cli(&args).await?;
            let url = output.trim().to_string();
            let number = url.split('/').last()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow!("Failed to extract PR number from URL"))?;
            
            Ok(PrRef { number, url })
        } else {
            let payload = serde_json::json!({
                "title": req.title,
                "body": body,
                "head": req.head,
                "base": req.base,
                "draft": req.draft,
            });
            
            let endpoint = format!("/repos/{}/{}/pulls", self.owner, self.repo);
            let resp = self.api_call("POST", &endpoint, Some(payload)).await?;
            
            Ok(PrRef {
                number: resp["number"].as_u64().ok_or_else(|| anyhow!("No PR number"))?,
                url: resp["html_url"].as_str().ok_or_else(|| anyhow!("No PR URL"))?.to_string(),
            })
        }
    }
    
    async fn update_pr(&self, pr: &PrRef, update: PrUpdate) -> Result<()> {
        let body = if let Some(ref body_text) = update.body {
            Some(self.prepare_body(body_text, update.stack_metadata.as_ref()))
        } else {
            None
        };
        
        if self.use_cli {
            let pr_str = pr.number.to_string();
            
            if let Some(ref title) = update.title {
                self.gh_cli(&["pr", "edit", &pr_str, "--title", title]).await?;
            }
            
            if let Some(ref body) = body {
                self.gh_cli(&["pr", "edit", &pr_str, "--body", body]).await?;
            }
            
            if let Some(ref base) = update.base {
                self.gh_cli(&["pr", "edit", &pr_str, "--base", base]).await?;
            }
            
            Ok(())
        } else {
            let mut payload = serde_json::Map::new();
            
            if let Some(title) = update.title {
                payload.insert("title".to_string(), Value::String(title));
            }
            
            if let Some(body) = body {
                payload.insert("body".to_string(), Value::String(body));
            }
            
            if let Some(base) = update.base {
                payload.insert("base".to_string(), Value::String(base));
            }
            
            let endpoint = format!("/repos/{}/{}/pulls/{}", self.owner, self.repo, pr.number);
            self.api_call("PATCH", &endpoint, Some(Value::Object(payload))).await?;
            
            Ok(())
        }
    }
    
    async fn list_prs(&self) -> Result<Vec<PrInfo>> {
        if self.use_cli {
            let output = self.gh_cli(&[
                "pr", "list",
                "--json", "number,title,url,state,isDraft,body",
            ]).await?;
            
            let prs: Vec<Value> = serde_json::from_str(&output)?;
            
            Ok(prs.into_iter().map(|pr| {
                let body = pr["body"].as_str().unwrap_or("");
                PrInfo {
                    pr_ref: PrRef {
                        number: pr["number"].as_u64().unwrap_or(0),
                        url: pr["url"].as_str().unwrap_or("").to_string(),
                    },
                    title: pr["title"].as_str().unwrap_or("").to_string(),
                    state: pr["state"].as_str().unwrap_or("").to_string(),
                    draft: pr["isDraft"].as_bool().unwrap_or(false),
                    stack_metadata: StackMetadata::parse_from_body(body),
                }
            }).collect())
        } else {
            let endpoint = format!("/repos/{}/{}/pulls", self.owner, self.repo);
            let resp = self.api_call("GET", &endpoint, None).await?;
            
            let prs = resp.as_array().ok_or_else(|| anyhow!("Expected array"))?;
            
            Ok(prs.iter().map(|pr| {
                let body = pr["body"].as_str().unwrap_or("");
                PrInfo {
                    pr_ref: PrRef {
                        number: pr["number"].as_u64().unwrap_or(0),
                        url: pr["html_url"].as_str().unwrap_or("").to_string(),
                    },
                    title: pr["title"].as_str().unwrap_or("").to_string(),
                    state: pr["state"].as_str().unwrap_or("").to_string(),
                    draft: pr["draft"].as_bool().unwrap_or(false),
                    stack_metadata: StackMetadata::parse_from_body(body),
                }
            }).collect())
        }
    }
    
    async fn get_pr_status(&self, pr: &PrRef) -> Result<PrStatus> {
        if self.use_cli {
            let pr_str = pr.number.to_string();
            let output = self.gh_cli(&[
                "pr", "view", &pr_str,
                "--json", "state,mergeable,statusCheckRollup",
            ]).await?;
            
            let data: Value = serde_json::from_str(&output)?;
            let checks = data["statusCheckRollup"].as_array();
            let checks_passing = checks.map(|checks| {
                checks.iter().all(|c| c["conclusion"].as_str() == Some("SUCCESS"))
            });
            
            Ok(PrStatus {
                state: data["state"].as_str().unwrap_or("").to_string(),
                mergeable: data["mergeable"].as_str().map(|s| s == "MERGEABLE"),
                checks_passing,
            })
        } else {
            let endpoint = format!("/repos/{}/{}/pulls/{}", self.owner, self.repo, pr.number);
            let data = self.api_call("GET", &endpoint, None).await?;
            
            Ok(PrStatus {
                state: data["state"].as_str().unwrap_or("").to_string(),
                mergeable: data["mergeable"].as_bool(),
                checks_passing: None,
            })
        }
    }
}
