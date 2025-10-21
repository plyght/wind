use crate::models::*;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CollabProvider {
    async fn create_pr(&self, req: CreatePrRequest) -> Result<PrRef>;
    
    async fn update_pr(&self, pr: &PrRef, update: PrUpdate) -> Result<()>;
    
    async fn list_prs(&self) -> Result<Vec<PrInfo>>;
    
    async fn get_pr_status(&self, pr: &PrRef) -> Result<PrStatus>;
}
