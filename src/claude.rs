use crate::error::{Error, Result};
use crate::http::HttpClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct ClaudeClient {
    api_key: String,
    model: String,
    max_tokens: u32,
    http: HttpClient,
}

#[derive(Serialize)]
struct MessageRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

impl ClaudeClient {
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Result<Self> {
        let http = HttpClient::new("st-narrative/0.1.0")?;
        Ok(Self {
            api_key,
            model,
            max_tokens,
            http,
        })
    }

    pub async fn complete(&self, system: &str, user_message: &str) -> Result<String> {
        let request = MessageRequest {
            model: &self.model,
            max_tokens: self.max_tokens,
            system,
            messages: vec![Message {
                role: "user",
                content: user_message,
            }],
        };

        let body = serde_json::to_string(&request)
            .map_err(|e| Error::parse(format!("serialize request: {e}")))?;

        debug!(model = %self.model, "sending Claude API request");

        let response_text = self
            .http
            .post_json_raw(
                CLAUDE_API_URL,
                &body,
                &[
                    ("x-api-key", &self.api_key),
                    ("anthropic-version", "2023-06-01"),
                ],
            )
            .await
            .map_err(|e| {
                warn!("Claude API error: {e}");
                e
            })?;

        let resp: MessageResponse = serde_json::from_str(&response_text)
            .map_err(|e| Error::parse(format!("parse Claude response: {e}")))?;

        let text = resp
            .content
            .into_iter()
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }

    /// Send a prompt and parse the response as JSON, stripping markdown fences if present.
    pub async fn complete_json<T: serde::de::DeserializeOwned>(
        &self,
        system: &str,
        user_message: &str,
    ) -> Result<T> {
        let text = self.complete(system, user_message).await?;
        let json_str = extract_json(&text);
        serde_json::from_str(json_str)
            .map_err(|e| Error::parse(format!("parse Claude JSON: {e}\nraw: {text}")))
    }
}

/// Extract JSON from a response that might be wrapped in markdown code fences.
fn extract_json(text: &str) -> &str {
    // Try to find JSON block in markdown fences
    if let Some(start) = text.find("```json") {
        let content = &text[start + 7..];
        if let Some(end) = content.find("```") {
            return content[..end].trim();
        }
    }
    if let Some(start) = text.find("```") {
        let content = &text[start + 3..];
        if let Some(end) = content.find("```") {
            let inner = content[..end].trim();
            if inner.starts_with('{') || inner.starts_with('[') {
                return inner;
            }
        }
    }
    // Try raw JSON
    if let Some(start) = text.find('{')
        && let Some(end) = text.rfind('}')
    {
        return &text[start..=end];
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_from_markdown() {
        let input = "Here's the result:\n```json\n{\"key\": \"value\"}\n```\n";
        assert_eq!(extract_json(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn extract_json_raw() {
        let input = "Sure! {\"key\": \"value\"} done.";
        assert_eq!(extract_json(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn extract_json_plain() {
        let input = "{\"key\": \"value\"}";
        assert_eq!(extract_json(input), "{\"key\": \"value\"}");
    }
}
