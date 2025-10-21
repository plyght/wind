use anyhow::Result;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

use super::{AiOpts, AiProvider};

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Content {
    text: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: usize,
    output_tokens: usize,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: "claude-3-sonnet-20240229".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    async fn complete(&self, prompt: &str, opts: AiOpts) -> Result<String> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: opts.max_tokens.unwrap_or(2000),
            temperature: opts.temperature,
            stream: false,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Anthropic API error: {}", error_text);
        }

        let data: AnthropicResponse = response.json().await?;

        if let Some(usage) = data.usage {
            let cost = self.cost_estimate(usage.input_tokens, usage.output_tokens);
            eprintln!(
                "Tokens: {} in, {} out (est. ${:.4})",
                usage.input_tokens, usage.output_tokens, cost
            );
        }

        Ok(data
            .content
            .first()
            .and_then(|c| c.text.clone())
            .unwrap_or_default())
    }

    async fn complete_stream(
        &self,
        prompt: &str,
        opts: AiOpts,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Unpin + Send>> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: opts.max_tokens.unwrap_or(2000),
            temperature: opts.temperature,
            stream: true,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let json_str = &line[6..];
                    if let Ok(data) = serde_json::from_str::<AnthropicResponse>(json_str) {
                        if let Some(content) = data.content.first() {
                            if let Some(text) = &content.text {
                                return Ok(text.clone());
                            }
                        }
                    }
                }
            }
            Ok(String::new())
        });

        Ok(Box::new(Box::pin(stream)))
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f64 / 4.0).ceil() as usize
    }

    fn cost_estimate(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;
        input_cost + output_cost
    }
}
