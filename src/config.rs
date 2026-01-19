use crate::provider::Provider;
use std::fs;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_loaded_config_default() {
        let config = LoadedConfig::default();
        assert!(config.system.is_none());
        assert!(config.prompt.is_none());
        assert!(config.file.is_none());
        assert!(config.image.is_none());
        assert!(config.video.is_none());
        assert!(config.audio.is_none());
        assert!(config.provider.is_none());
        assert!(config.model.is_none());
        assert!(config.temperature.is_none());
        assert!(config.max_tokens.is_none());
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_loaded_config_parse_system() {
        let content = "---system---\nYou are a helpful assistant.";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(
            config.system,
            Some("You are a helpful assistant.".to_string())
        );
    }

    #[test]
    fn test_loaded_config_parse_prompt() {
        let content = "---prompt---\nHello, world!";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.prompt, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_loaded_config_parse_provider_openai() {
        let content = "---provider---\nopenai";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert!(matches!(config.provider, Some(Provider::Openai)));
    }

    #[test]
    fn test_loaded_config_parse_provider_anthropic() {
        let content = "---provider---\nAnthropic";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert!(matches!(config.provider, Some(Provider::Anthropic)));
    }

    #[test]
    fn test_loaded_config_parse_provider_ollama() {
        let content = "---provider---\nOLLAMA";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert!(matches!(config.provider, Some(Provider::Ollama)));
    }

    #[test]
    fn test_loaded_config_parse_provider_openrouter() {
        let content = "---provider---\nopenrouter";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert!(matches!(config.provider, Some(Provider::Openrouter)));
    }

    #[test]
    fn test_loaded_config_parse_temperature() {
        let content = "---temperature---\n0.5";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.temperature, Some(0.5));
    }

    #[test]
    fn test_loaded_config_parse_max_tokens() {
        let content = "---max_tokens---\n2048";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.max_tokens, Some(2048));
    }

    #[test]
    fn test_loaded_config_parse_model() {
        let content = "---model---\ngpt-4-turbo";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.model, Some("gpt-4-turbo".to_string()));
    }

    #[test]
    fn test_loaded_config_parse_base_url() {
        let content = "---base_url---\nhttps://custom.api.com/v1";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(
            config.base_url,
            Some("https://custom.api.com/v1".to_string())
        );
    }

    #[test]
    fn test_loaded_config_parse_file_path() {
        let content = "---file---\n/path/to/file.txt";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.file, Some(PathBuf::from("/path/to/file.txt")));
    }

    #[test]
    fn test_loaded_config_parse_image_path() {
        let content = "---image---\n/path/to/image.png";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.image, Some(PathBuf::from("/path/to/image.png")));
    }

    #[test]
    fn test_loaded_config_parse_multiple_sections() {
        let content = r#"---system---
You are helpful.

---prompt---
Test prompt

---temperature---
0.8

---max_tokens---
1024"#;
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();

        assert_eq!(config.system, Some("You are helpful.".to_string()));
        assert_eq!(config.prompt, Some("Test prompt".to_string()));
        assert_eq!(config.temperature, Some(0.8));
        assert_eq!(config.max_tokens, Some(1024));
    }

    #[test]
    fn test_loaded_config_multiline_prompt() {
        let content = r#"---prompt---
Line 1
Line 2
Line 3"#;
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.prompt, Some("Line 1\nLine 2\nLine 3".to_string()));
    }

    #[test]
    fn test_loaded_config_unknown_section_ignored() {
        let content = "---unknown---\nsome value\n---prompt---\ntest";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.prompt, Some("test".to_string()));
    }

    #[test]
    fn test_loaded_config_invalid_temperature_ignored() {
        let content = "---temperature---\ninvalid";
        let file = create_temp_config(content);
        let config = LoadedConfig::from_file(&file.path().to_path_buf()).unwrap();
        assert!(config.temperature.is_none());
    }

    #[test]
    fn test_merged_config_without_loaded() {
        let provider = Provider::Openai;
        let merged = MergedConfig::new(
            None,
            &provider,
            Some("gpt-4"),
            Some("https://api.example.com"),
            Some("System prompt"),
            0.9,
            Some(500),
        );

        assert_eq!(merged.model, Some("gpt-4"));
        assert_eq!(merged.base_url, Some("https://api.example.com"));
        assert_eq!(merged.system, Some("System prompt"));
        assert_eq!(merged.temperature, 0.9);
        assert_eq!(merged.max_tokens, Some(500));
        assert!(matches!(merged.provider, Provider::Openai));
    }

    #[test]
    fn test_merged_config_with_loaded_overrides() {
        let mut loaded = LoadedConfig::default();
        loaded.model = Some("claude-sonnet-4-20250514".to_string());
        loaded.temperature = Some(0.5);
        loaded.provider = Some(Provider::Anthropic);

        let cli_provider = Provider::Openai;
        let merged = MergedConfig::new(
            Some(&loaded),
            &cli_provider,
            Some("gpt-4"),
            None,
            None,
            0.7,
            None,
        );

        assert_eq!(merged.model, Some("claude-sonnet-4-20250514"));
        assert_eq!(merged.temperature, 0.5);
        assert!(matches!(merged.provider, Provider::Anthropic));
    }

    #[test]
    fn test_merged_config_cli_fallback() {
        let loaded = LoadedConfig::default();

        let cli_provider = Provider::Ollama;
        let merged = MergedConfig::new(
            Some(&loaded),
            &cli_provider,
            Some("gemma3"),
            Some("http://localhost:11434"),
            Some("Be helpful"),
            0.6,
            Some(1000),
        );

        assert_eq!(merged.model, Some("gemma3"));
        assert_eq!(merged.base_url, Some("http://localhost:11434"));
        assert_eq!(merged.system, Some("Be helpful"));
        assert_eq!(merged.temperature, 0.6);
        assert_eq!(merged.max_tokens, Some(1000));
    }

    #[test]
    fn test_merged_config_model_name_with_model() {
        let provider = Provider::Openai;
        let merged =
            MergedConfig::new(None, &provider, Some("custom-model"), None, None, 0.7, None);
        assert_eq!(merged.model_name(), "custom-model");
    }

    #[test]
    fn test_merged_config_model_name_default_fallback() {
        let provider = Provider::Openai;
        let merged = MergedConfig::new(None, &provider, None, None, None, 0.7, None);
        assert_eq!(merged.model_name(), "gpt-4o");
    }

    #[test]
    fn test_merged_config_has_media_false() {
        let provider = Provider::Openai;
        let merged = MergedConfig::new(None, &provider, None, None, None, 0.7, None);
        assert!(!merged.has_media());
    }

    #[test]
    fn test_merged_config_has_media_with_image() {
        let mut loaded = LoadedConfig::default();
        loaded.image = Some(PathBuf::from("/path/to/image.jpg"));

        let provider = Provider::Openai;
        let merged = MergedConfig::new(Some(&loaded), &provider, None, None, None, 0.7, None);
        assert!(merged.has_media());
    }

    #[test]
    fn test_merged_config_has_media_with_video() {
        let mut loaded = LoadedConfig::default();
        loaded.video = Some(PathBuf::from("/path/to/video.mp4"));

        let provider = Provider::Openai;
        let merged = MergedConfig::new(Some(&loaded), &provider, None, None, None, 0.7, None);
        assert!(merged.has_media());
    }

    #[test]
    fn test_merged_config_has_media_with_audio() {
        let mut loaded = LoadedConfig::default();
        loaded.audio = Some(PathBuf::from("/path/to/audio.mp3"));

        let provider = Provider::Openai;
        let merged = MergedConfig::new(Some(&loaded), &provider, None, None, None, 0.7, None);
        assert!(merged.has_media());
    }

    #[test]
    fn test_template_contains_all_sections() {
        assert!(TEMPLATE.contains("---provider---"));
        assert!(TEMPLATE.contains("---model---"));
        assert!(TEMPLATE.contains("---base_url---"));
        assert!(TEMPLATE.contains("---temperature---"));
        assert!(TEMPLATE.contains("---max_tokens---"));
        assert!(TEMPLATE.contains("---system---"));
        assert!(TEMPLATE.contains("---prompt---"));
        assert!(TEMPLATE.contains("---file---"));
        assert!(TEMPLATE.contains("---image---"));
        assert!(TEMPLATE.contains("---video---"));
        assert!(TEMPLATE.contains("---audio---"));
    }
}
