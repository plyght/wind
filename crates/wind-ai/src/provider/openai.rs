use anyhow::Result;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

use super::{AiOpts, AiProvider};

pub struct OpenAiProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: Option<Message>,
    delta: Option<Delta>,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: "gpt-4".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn complete(&self, prompt: &str, opts: AiOpts) -> Result<String> {
        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: opts.max_tokens,
            temperature: opts.temperature,
            stream: false,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("OpenAI API error: {}", error_text);
        }

        let data: OpenAiResponse = response.json().await?;

        if let Some(usage) = data.usage {
            let cost = self.cost_estimate(usage.prompt_tokens, usage.completion_tokens);
            eprintln!(
                "Tokens: {} in, {} out (est. ${:.4})",
                usage.prompt_tokens, usage.completion_tokens, cost
            );
        }

        Ok(data.choices[0]
            .message
            .as_ref()
            .map(|m| m.content.clone())
            .unwrap_or_default())
    }

    async fn complete_stream(
        &self,
        prompt: &str,
        opts: AiOpts,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Unpin + Send>> {
        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: opts.max_tokens,
            temperature: opts.temperature,
            stream: true,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let json_str = &line[6..];
                    if json_str == "[DONE]" {
                        continue;
                    }
                    if let Ok(data) = serde_json::from_str::<OpenAiResponse>(json_str) {
                        if let Some(choice) = data.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    return Ok(content.clone());
                                }
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
        let input_cost = (input_tokens as f64 / 1000.0) * 0.03;
        let output_cost = (output_tokens as f64 / 1000.0) * 0.06;
        input_cost + output_cost
    }
}
