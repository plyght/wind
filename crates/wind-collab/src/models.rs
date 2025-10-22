use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrRef {
    pub number: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    pub pr_ref: PrRef,
    pub title: String,
    pub state: String,
    pub draft: bool,
    pub stack_metadata: Option<StackMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrStatus {
    pub state: String,
    pub mergeable: Option<bool>,
    pub checks_passing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackMetadata {
    pub parent_pr: Option<PrRef>,
    pub child_prs: Vec<PrRef>,
    pub stack_position: usize,
    pub stack_size: usize,
}

impl StackMetadata {
    pub fn serialize_for_body(&self) -> String {
        let mut parts = vec![
            "<!-- WIND_STACK_METADATA".to_string(),
            serde_json::to_string(self).unwrap_or_default(),
            "-->".to_string(),
        ];
        
        if let Some(parent) = &self.parent_pr {
            parts.push(format!(
                "\n\n**Stack:** Part {}/{} | Parent: #{}",
                self.stack_position, self.stack_size, parent.number
            ));
            parts.push(format!("Parent PR: {}", parent.url));
        } else {
            parts.push(format!("\n\n**Stack:** Part {}/{} (Base)", self.stack_position, self.stack_size));
        }
        
        parts.join("\n")
    }
    
    pub fn parse_from_body(body: &str) -> Option<Self> {
        let start = body.find("<!-- WIND_STACK_METADATA")?;
        let end = body[start..].find("-->")?;
        let json_start = start + "<!-- WIND_STACK_METADATA\n".len();
        let json_str = &body[json_start..start + end];
        serde_json::from_str(json_str).ok()
    }
}

#[derive(Debug, Clone)]
pub struct CreatePrRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
    pub draft: bool,
    pub stack_metadata: Option<StackMetadata>,
}

#[derive(Debug, Clone)]
pub struct PrUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub base: Option<String>,
    pub stack_metadata: Option<StackMetadata>,
}
