use crate::mcp::{LoggingMcpTool, McpSession};
use crate::provider::Provider;
use crate::ui::{status, status_done_detail};
use rig::completion::Prompt;
use rig::completion::{AssistantContent, CompletionModel};
use rig::prelude::*;
use rig::providers::{anthropic, ollama, openai, openrouter};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub base_url: Option<String>,
    pub mcp_sessions: Vec<McpSession>,
    pub max_turns: usize,
}

impl ChatConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            temperature: 0.7,
            max_turns: 10,
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

    pub fn base_url(mut self, url: Option<String>) -> Self {
        self.base_url = url;
        self
    }

    pub fn mcp_sessions(mut self, sessions: Vec<McpSession>) -> Self {
        self.mcp_sessions = sessions;
        self
    }

    pub fn max_turns(mut self, turns: usize) -> Self {
        self.max_turns = turns;
        self
    }

    pub fn has_media(&self) -> bool {
        self.image.is_some() || self.video.is_some() || self.audio.is_some()
    }

    pub fn has_mcp(&self) -> bool {
        !self.mcp_sessions.is_empty()
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

    if config.has_mcp() {
        return send_with_mcp(provider, prompt, &config).await;
    }

    send_text_only(provider, prompt, &config).await
}

async fn send_with_mcp(
    provider: &Provider,
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    if config.mcp_sessions.is_empty() {
        return Err("No MCP sessions configured".into());
    }

    let total_tools: usize = config.mcp_sessions.iter().map(|s| s.tools.len()).sum();
    status_done_detail("MCP tools loaded", &format!("{} tools", total_tools));

    let called_tools: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    use rig::tool::server::ToolServer;
    let tool_server = ToolServer::new();
    let tool_server_handle = tool_server.run();

    for session in &config.mcp_sessions {
        for t in &session.tools {
            let logging_tool: LoggingMcpTool =
                LoggingMcpTool::new(t.clone(), session.peer.clone(), called_tools.clone());
            tool_server_handle
                .add_tool(logging_tool)
                .await
                .map_err(|e| format!("Failed to add tool: {:?}", e))?;
        }
    }

    status("Sending request...");

    let tool_names: Vec<String> = config
        .mcp_sessions
        .iter()
        .flat_map(|s| s.tools.iter().map(|t| t.name.to_string()))
        .collect();
    let tool_hint = format!(
        "\nYou have access to the following tools: {}.\nIf the user asks for something that requires these tools, you MUST use them.",
        tool_names.join(", ")
    );

    macro_rules! build_agent_with_tools {
        ($client:expr, $tool_server_handle:expr, $called_tools:expr) => {{
            let mut builder = $client.agent(&config.model).temperature(config.temperature);

            let mut preamble = config.system.clone().unwrap_or_default();
            preamble.push_str(&tool_hint);
            builder = builder.preamble(&preamble);
            if let Some(tokens) = config.max_tokens {
                builder = builder.max_tokens(tokens);
            }

            let result: Result<String, Box<dyn std::error::Error>> = {
                let agent = builder.tool_server_handle($tool_server_handle).build();

                let response = agent.prompt(prompt).max_turns(config.max_turns).await?;

                let tools_used = $called_tools.lock().await;
                if !tools_used.is_empty() {
                    eprintln!("[+] Tools called: {}", tools_used.join(" → "));
                }

                Ok(response)
            };

            result
        }};
    }

    match provider {
        Provider::Openai => {
            build_agent_with_tools!(openai::Client::from_env(), tool_server_handle, called_tools)
        }
        Provider::Anthropic => {
            build_agent_with_tools!(
                anthropic::Client::from_env(),
                tool_server_handle,
                called_tools
            )
        }
        Provider::Ollama => {
            build_agent_with_tools!(ollama::Client::from_env(), tool_server_handle, called_tools)
        }
        Provider::Openrouter => {
            build_agent_with_tools!(
                openrouter::Client::from_env(),
                tool_server_handle,
                called_tools
            )
        }
    }
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
        Provider::Openai => send_text_openai(prompt, config).await,
        Provider::Anthropic => build_and_send!(anthropic::Client::from_env()),
        Provider::Ollama => build_and_send!(ollama::Client::from_env()),
        Provider::Openrouter => build_and_send!(openrouter::Client::from_env()),
    }
}

async fn send_text_openai(
    prompt: &str,
    config: &ChatConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = openai::Client::from_env();
    let model = client.completion_model(&config.model);

    let mut builder = model
        .completion_request(prompt)
        .temperature(config.temperature);

    if let Some(sys) = &config.system {
        builder = builder.preamble(sys.clone());
    }
    if let Some(tokens) = config.max_tokens {
        builder = builder.max_tokens(tokens);
    }

    let response = builder.send().await?;

    match response.choice.first() {
        AssistantContent::Text(text) => Ok(text.text.clone()),
        AssistantContent::ToolCall(tc) => {
            Err(format!("Unexpected tool call: {}", tc.function.name).into())
        }
        _ => Err("Unexpected response type".into()),
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

    let base_url = config
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let url = format!("{}/chat/completions", base_url);

    let response: serde_json::Value = client
        .post(&url)
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

    let base_url = config
        .base_url
        .as_deref()
        .unwrap_or("https://api.anthropic.com");
    let url = format!("{}/v1/messages", base_url);

    let response: serde_json::Value = client
        .post(&url)
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

    let base_url = config
        .base_url
        .as_deref()
        .unwrap_or("https://openrouter.ai/api/v1");
    let url = format!("{}/chat/completions", base_url);

    let response: serde_json::Value = client
        .post(&url)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_config_new() {
        let config = ChatConfig::new("gpt-4");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature, 0.7);
        assert!(config.system.is_none());
        assert!(config.max_tokens.is_none());
        assert!(config.image.is_none());
        assert!(config.video.is_none());
        assert!(config.audio.is_none());
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_chat_config_default() {
        let config = ChatConfig::default();
        assert!(config.model.is_empty());
        assert_eq!(config.temperature, 0.0);
        assert!(config.system.is_none());
    }

    #[test]
    fn test_chat_config_builder_system() {
        let config = ChatConfig::new("model").system(Some("You are helpful".to_string()));
        assert_eq!(config.system, Some("You are helpful".to_string()));
    }

    #[test]
    fn test_chat_config_builder_temperature() {
        let config = ChatConfig::new("model").temperature(0.9);
        assert_eq!(config.temperature, 0.9);
    }

    #[test]
    fn test_chat_config_builder_max_tokens() {
        let config = ChatConfig::new("model").max_tokens(Some(2048));
        assert_eq!(config.max_tokens, Some(2048));
    }

    #[test]
    fn test_chat_config_builder_image() {
        let config = ChatConfig::new("model").image(Some("base64data".to_string()));
        assert_eq!(config.image, Some("base64data".to_string()));
    }

    #[test]
    fn test_chat_config_builder_video() {
        let config = ChatConfig::new("model").video(Some("video_data".to_string()));
        assert_eq!(config.video, Some("video_data".to_string()));
    }

    #[test]
    fn test_chat_config_builder_audio() {
        let config = ChatConfig::new("model").audio(Some("audio_data".to_string()));
        assert_eq!(config.audio, Some("audio_data".to_string()));
    }

    #[test]
    fn test_chat_config_builder_base_url() {
        let config = ChatConfig::new("model").base_url(Some("https://custom.api.com".to_string()));
        assert_eq!(config.base_url, Some("https://custom.api.com".to_string()));
    }

    #[test]
    fn test_chat_config_builder_chain() {
        let config = ChatConfig::new("gpt-4")
            .system(Some("Be helpful".to_string()))
            .temperature(0.8)
            .max_tokens(Some(1000))
            .base_url(Some("https://api.test.com".to_string()));

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.system, Some("Be helpful".to_string()));
        assert_eq!(config.temperature, 0.8);
        assert_eq!(config.max_tokens, Some(1000));
        assert_eq!(config.base_url, Some("https://api.test.com".to_string()));
    }

    #[test]
    fn test_chat_config_has_media_false() {
        let config = ChatConfig::new("model");
        assert!(!config.has_media());
    }

    #[test]
    fn test_chat_config_has_media_with_image() {
        let config = ChatConfig::new("model").image(Some("data".to_string()));
        assert!(config.has_media());
    }

    #[test]
    fn test_chat_config_has_media_with_video() {
        let config = ChatConfig::new("model").video(Some("data".to_string()));
        assert!(config.has_media());
    }

    #[test]
    fn test_chat_config_has_media_with_audio() {
        let config = ChatConfig::new("model").audio(Some("data".to_string()));
        assert!(config.has_media());
    }

    #[test]
    fn test_chat_config_has_media_multiple() {
        let config = ChatConfig::new("model")
            .image(Some("img".to_string()))
            .audio(Some("aud".to_string()));
        assert!(config.has_media());
    }

    #[test]
    fn test_chat_config_has_mcp_false() {
        let config = ChatConfig::new("model");
        assert!(!config.has_mcp());
    }

    #[test]
    fn test_chat_config_mcp_session_builder() {
        let config = ChatConfig::new("model").mcp_sessions(vec![]);
        assert!(!config.has_mcp());
    }

    #[test]
    fn test_build_vision_content_text_only() {
        let content = build_vision_content("Hello", &None, true);
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "Hello");
    }

    #[test]
    fn test_build_vision_content_openai_format_with_image() {
        let image = Some("base64data".to_string());
        let content = build_vision_content("Describe", &image, true);
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "image_url");
        assert!(
            arr[0]["image_url"]["url"]
                .as_str()
                .unwrap()
                .contains("base64data")
        );
    }

    #[test]
    fn test_build_vision_content_anthropic_format_with_image() {
        let image = Some("base64data".to_string());
        let content = build_vision_content("Describe", &image, false);
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "image");
        assert_eq!(arr[0]["source"]["type"], "base64");
        assert_eq!(arr[0]["source"]["data"], "base64data");
    }

    #[test]
    fn test_build_openai_payload() {
        let content = json!([{"type": "text", "text": "test"}]);
        let payload = build_openai_payload("gpt-4", content, 0.5);

        assert_eq!(payload["model"], "gpt-4");
        assert_eq!(payload["temperature"], 0.5);
        assert!(payload["messages"].is_array());
        assert_eq!(payload["messages"][0]["role"], "user");
    }

    #[test]
    fn test_insert_system_message() {
        let content = json!([{"type": "text", "text": "test"}]);
        let mut payload = build_openai_payload("gpt-4", content, 0.5);

        insert_system_message(&mut payload, "You are helpful");

        let messages = payload["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are helpful");
        assert_eq!(messages[1]["role"], "user");
    }

    #[test]
    fn test_extract_openai_response_valid() {
        let response = json!({
            "choices": [{
                "message": {
                    "content": "Hello, world!"
                }
            }]
        });
        let result = extract_openai_response(&response).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_extract_openai_response_missing_content() {
        let response = json!({
            "choices": [{
                "message": {}
            }]
        });
        let result = extract_openai_response(&response).unwrap();
        assert_eq!(result, "No response");
    }

    #[test]
    fn test_extract_openai_response_empty() {
        let response = json!({});
        let result = extract_openai_response(&response).unwrap();
        assert_eq!(result, "No response");
    }
}
