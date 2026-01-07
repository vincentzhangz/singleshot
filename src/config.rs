use std::fs;
use std::path::PathBuf;

use crate::provider::Provider;

#[derive(Default)]
pub struct LoadedConfig {
    pub system: Option<String>,
    pub prompt: Option<String>,
    pub file: Option<PathBuf>,
    pub image: Option<PathBuf>,
    pub video: Option<PathBuf>,
    pub audio: Option<PathBuf>,
    pub provider: Option<Provider>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
    pub base_url: Option<String>,
}

impl LoadedConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Self::default();

        let mut current_section: Option<String> = None;
        let mut current_value = String::new();

        for line in content.lines() {
            if line.starts_with("---") && line.ends_with("---") && line.len() > 6 {
                if let Some(section) = current_section.take() {
                    let value = current_value.trim();
                    if !value.is_empty() {
                        config.parse_section(&section, value);
                    }
                }
                current_section = Some(line[3..line.len() - 3].to_string());
                current_value.clear();
            } else if current_section.is_some() {
                if !current_value.is_empty() {
                    current_value.push('\n');
                }
                current_value.push_str(line);
            }
        }

        if let Some(section) = current_section {
            let value = current_value.trim();
            if !value.is_empty() {
                config.parse_section(&section, value);
            }
        }

        Ok(config)
    }

    fn parse_section(&mut self, section: &str, value: &str) {
        match section {
            "system" => self.system = Some(value.to_string()),
            "prompt" => self.prompt = Some(value.to_string()),
            "file" => self.file = Some(PathBuf::from(value)),
            "image" => self.image = Some(PathBuf::from(value)),
            "video" => self.video = Some(PathBuf::from(value)),
            "audio" => self.audio = Some(PathBuf::from(value)),
            "model" => self.model = Some(value.to_string()),
            "base_url" => self.base_url = Some(value.to_string()),
            "provider" => {
                self.provider = match value.to_lowercase().as_str() {
                    "openai" => Some(Provider::Openai),
                    "anthropic" => Some(Provider::Anthropic),
                    "ollama" => Some(Provider::Ollama),
                    "openrouter" => Some(Provider::Openrouter),
                    _ => None,
                };
            }
            "temperature" => self.temperature = value.parse().ok(),
            "max_tokens" => self.max_tokens = value.parse().ok(),
            _ => {}
        }
    }
}

#[allow(dead_code)]
pub struct MergedConfig<'a> {
    pub prompt: Option<String>,
    pub file: Option<PathBuf>,
    pub system: Option<&'a str>,
    pub image: Option<PathBuf>,
    pub video: Option<PathBuf>,
    pub audio: Option<PathBuf>,
    pub provider: &'a Provider,
    pub model: Option<&'a str>,
    pub temperature: f64,
    pub max_tokens: Option<u64>,
    pub base_url: Option<&'a str>,
}

impl<'a> MergedConfig<'a> {
    pub fn new(
        loaded: Option<&'a LoadedConfig>,
        cli_provider: &'a Provider,
        cli_model: Option<&'a str>,
        cli_base_url: Option<&'a str>,
        cli_system: Option<&'a str>,
        cli_temperature: f64,
        cli_max_tokens: Option<u64>,
    ) -> Self {
        if let Some(cfg) = loaded {
            Self {
                prompt: cfg.prompt.clone(),
                file: cfg.file.clone(),
                system: cfg.system.as_deref().or(cli_system),
                image: cfg.image.clone(),
                video: cfg.video.clone(),
                audio: cfg.audio.clone(),
                provider: cfg.provider.as_ref().unwrap_or(cli_provider),
                model: cfg.model.as_deref().or(cli_model),
                temperature: cfg.temperature.unwrap_or(cli_temperature),
                max_tokens: cfg.max_tokens.or(cli_max_tokens),
                base_url: cfg.base_url.as_deref().or(cli_base_url),
            }
        } else {
            Self {
                prompt: None,
                file: None,
                system: cli_system,
                image: None,
                video: None,
                audio: None,
                provider: cli_provider,
                model: cli_model,
                temperature: cli_temperature,
                max_tokens: cli_max_tokens,
                base_url: cli_base_url,
            }
        }
    }

    pub fn model_name(&self) -> String {
        self.model
            .map(String::from)
            .unwrap_or_else(|| self.provider.default_model().to_string())
    }

    #[allow(dead_code)]
    pub fn has_media(&self) -> bool {
        self.image.is_some() || self.video.is_some() || self.audio.is_some()
    }
}

pub const TEMPLATE: &str = r#"---provider---
openai

---model---
gpt-4

---base_url---
https://api.openai.com/v1

---temperature---
0.7

---max_tokens---
1024

---system---
You are a helpful AI assistant.

---prompt---
Please help me with my question.

---file---
path/to/your/file.txt

---image---
path/to/your/image.jpg

---video---
path/to/your/video.mp4

---audio---
path/to/your/audio.mp3
"#;
