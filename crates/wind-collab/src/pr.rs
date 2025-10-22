use anyhow::Result;

pub struct PullRequest {
    pub number: u32,
    pub title: String,
    pub state: String,
    pub url: String,
}

pub async fn create(_title: Option<String>, _body: Option<String>) -> Result<PullRequest> {
    Ok(PullRequest {
        number: 1,
        title: "Example PR".to_string(),
        state: "open".to_string(),
        url: "https://github.com/example/repo/pull/1".to_string(),
    })
}

pub async fn update(_number: u32) -> Result<()> {
    Ok(())
}

pub async fn list() -> Result<Vec<PullRequest>> {
    Ok(vec![])
}
