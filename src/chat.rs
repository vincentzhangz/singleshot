use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::{anthropic, ollama, openai, openrouter};
use serde_json::json;

use crate::provider::Provider;

fn get_api_key(env_var: &str) -> Result<String, Box<dyn std::error::Error>> {
    std::env::var(env_var).map_err(|_| {
        format!(
            "Missing {} environment variable. Set it with:\n  export {}=\"your-api-key\"",
            env_var, env_var
        )
        .into()
    })
}

#[derive(Default)]
pub struct ChatConfig {
    pub model: String,
    pub system: Option<String>,
    pub temperature: f64,
    pub max_tokens: Option<u64>,
    pub image: Option<String>,
    pub video: Option<String>,
    pub audio: Option<String>,
}

impl ChatConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            temperature: 0.7,
            ..Default::default()
        }
    }

    pub fn system(mut self, system: Option<String>) -> Self {
        self.system = system;
        self
    }

    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = temp;
        self
    }

    pub fn max_tokens(mut self, tokens: Option<u64>) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn image(mut self, data: Option<String>) -> Self {
        self.image = data;
        self
    }

    pub fn video(mut self, data: Option<String>) -> Self {
        self.video = data;
        self
    }

    pub fn audio(mut self, data: Option<String>) -> Self {
        self.audio = data;
        self
    }

    pub fn has_media(&self) -> bool {
        self.image.is_some() || self.video.is_some() || self.audio.is_some()
    }
}

pub async fn send_prompt(
    provider: &Provider,
    prompt: &str,
    config: ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    if config.has_media() {
        return send_with_vision(provider, prompt, &config).await;
    }

    send_text_only(provider, prompt, &config).await
}

async fn send_text_only(
    provider: &Provider,
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    macro_rules! build_and_send {
        ($client:expr) => {{
            let mut builder = $client.agent(&config.model).temperature(config.temperature);

            if let Some(sys) = &config.system {
                builder = builder.preamble(sys);
            }
            if let Some(tokens) = config.max_tokens {
                builder = builder.max_tokens(tokens);
            }

            builder.build().prompt(prompt).await.map_err(Into::into)
        }};
    }

    match provider {
        Provider::Openai => build_and_send!(openai::Client::from_env()),
        Provider::Anthropic => build_and_send!(anthropic::Client::from_env()),
        Provider::Ollama => build_and_send!(ollama::Client::from_env()),
        Provider::Openrouter => build_and_send!(openrouter::Client::from_env()),
    }
}

async fn send_with_vision(
    provider: &Provider,
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    match provider {
        Provider::Openai => send_vision_openai(prompt, config).await,
        Provider::Anthropic => send_vision_anthropic(prompt, config).await,
        Provider::Openrouter => send_vision_openrouter(prompt, config).await,
        Provider::Ollama => {
            eprintln!("Warning: Ollama may not support vision. Sending as text.");
            send_text_only(provider, prompt, config).await
        }
    }
}

async fn send_vision_openai(
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = get_api_key("OPENAI_API_KEY")?;
    let client = reqwest::Client::new();

    let content = build_vision_content(prompt, &config.image, true);
    let mut payload = build_openai_payload(&config.model, content, config.temperature);

    if let Some(sys) = &config.system {
        insert_system_message(&mut payload, sys);
    }
    if let Some(tokens) = config.max_tokens {
        payload["max_tokens"] = json!(tokens);
    }

    let response: serde_json::Value = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    extract_openai_response(&response)
}

async fn send_vision_anthropic(
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = get_api_key("ANTHROPIC_API_KEY")?;
    let client = reqwest::Client::new();

    let content = build_vision_content(prompt, &config.image, false);
    let mut payload = json!({
        "model": config.model,
        "max_tokens": config.max_tokens.unwrap_or(1024),
        "temperature": config.temperature,
        "messages": [{"role": "user", "content": content}],
    });

    if let Some(sys) = &config.system {
        payload["system"] = json!(sys);
    }

    let response: serde_json::Value = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(response["content"][0]["text"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}

async fn send_vision_openrouter(
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = get_api_key("OPENROUTER_API_KEY")?;
    let client = reqwest::Client::new();

    let content = build_vision_content(prompt, &config.image, true);
    let mut payload = build_openai_payload(&config.model, content, config.temperature);

    if let Some(sys) = &config.system {
        insert_system_message(&mut payload, sys);
    }
    if let Some(tokens) = config.max_tokens {
        payload["max_tokens"] = json!(tokens);
    }

    let response: serde_json::Value = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    extract_openai_response(&response)
}

fn build_vision_content(
    prompt: &str,
    image: &Option<String>,
    openai_format: bool,
) -> serde_json::Value {
    let mut content = vec![json!({"type": "text", "text": prompt})];

    if let Some(base64_data) = image {
        let image_content = if openai_format {
            json!({
                "type": "image_url",
                "image_url": {"url": format!("data:image/jpeg;base64,{}", base64_data)}
            })
        } else {
            json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/jpeg",
                    "data": base64_data
                }
            })
        };
        content.insert(0, image_content);
    }

    json!(content)
}

fn build_openai_payload(
    model: &str,
    content: serde_json::Value,
    temperature: f64,
) -> serde_json::Value {
    json!({
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "temperature": temperature,
    })
}

fn insert_system_message(payload: &mut serde_json::Value, system: &str) {
    if let Some(messages) = payload["messages"].as_array_mut() {
        messages.insert(0, json!({"role": "system", "content": system}));
    }
}

fn extract_openai_response(
    response: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}
