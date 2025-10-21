use anyhow::Result;
use async_trait::async_trait;

pub mod anthropic;
pub mod openai;

pub use anthropic::AnthropicProvider;
pub use openai::OpenAiProvider;

#[derive(Debug, Clone)]
pub struct AiOpts {
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

impl Default for AiOpts {
    fn default() -> Self {
        Self {
            max_tokens: Some(2000),
            temperature: Some(0.7),
            stream: false,
        }
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, prompt: &str, opts: AiOpts) -> Result<String>;

    async fn complete_stream(
        &self,
        prompt: &str,
        opts: AiOpts,
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Unpin + Send>>;

    fn estimate_tokens(&self, text: &str) -> usize;

    fn cost_estimate(&self, input_tokens: usize, output_tokens: usize) -> f64;
}

pub fn get_provider() -> Result<Box<dyn AiProvider>> {
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            return Ok(Box::new(OpenAiProvider::new(key)));
        }
    }

    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() {
            return Ok(Box::new(AnthropicProvider::new(key)));
        }
    }

    anyhow::bail!("No AI provider API key found. Set OPENAI_API_KEY or ANTHROPIC_API_KEY")
}
