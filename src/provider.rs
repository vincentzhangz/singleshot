use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum Provider {
    Openai,
    Anthropic,
    #[default]
    Ollama,
    Openrouter,
}

impl Provider {
    pub const fn default_base_url(&self) -> &'static str {
        match self {
            Self::Openai => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com",
            Self::Ollama => "http://localhost:11434",
            Self::Openrouter => "https://openrouter.ai/api/v1",
        }
    }

    pub const fn default_model(&self) -> &'static str {
        match self {
            Self::Openai => "gpt-4o",
            Self::Anthropic => "claude-sonnet-4-20250514",
            Self::Ollama => "llama3.2",
            Self::Openrouter => "openai/gpt-4o",
        }
    }

    pub const fn ping_model(&self) -> &'static str {
        match self {
            Self::Openai => "gpt-4o-mini",
            Self::Anthropic => "claude-sonnet-4-5",
            Self::Ollama => "llama3.2",
            Self::Openrouter => "openai/gpt-4o-mini",
        }
    }

    #[allow(dead_code)]
    pub const fn env_var_name(&self) -> &'static str {
        match self {
            Self::Openai => "OPENAI_BASE_URL",
            Self::Ollama => "OLLAMA_API_BASE_URL",
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::Openrouter => "OPENROUTER_API_KEY",
        }
    }

    pub fn setup_base_url(&self, base_url: Option<&str>) {
        let url = base_url.unwrap_or(self.default_base_url());

        match self {
            Self::Openai => unsafe { std::env::set_var("OPENAI_BASE_URL", url) },
            Self::Ollama => unsafe { std::env::set_var("OLLAMA_API_BASE_URL", url) },
            Self::Anthropic | Self::Openrouter => {
                if base_url.is_some() {
                    eprintln!("Warning: Custom base URL for {:?} is not supported", self);
                }
            }
        }
    }

    pub async fn fetch_models(
        &self,
        base_url: Option<&str>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = base_url.unwrap_or(self.default_base_url());

        match self {
            Self::Openai => Self::fetch_openai_models(&client, url).await,
            Self::Anthropic => Self::fetch_anthropic_models(&client, url).await,
            Self::Ollama => Self::fetch_ollama_models(&client, url).await,
            Self::Openrouter => Self::fetch_openrouter_models(&client, url).await,
        }
    }

    async fn fetch_openai_models(
        client: &reqwest::Client,
        base_url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let response: serde_json::Value = client
            .get(format!("{}/models", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?
            .json()
            .await?;

        let models = response["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str())
                    .filter(|id| {
                        id.starts_with("gpt-")
                            || id.starts_with("o1")
                            || id.starts_with("o3")
                            || id.starts_with("o4")
                    })
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    async fn fetch_anthropic_models(
        client: &reqwest::Client,
        base_url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")?;
        let response: serde_json::Value = client
            .get(format!("{}/v1/models", base_url))
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await?
            .json()
            .await?;

        let models = response["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    async fn fetch_ollama_models(
        client: &reqwest::Client,
        base_url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response: serde_json::Value = client
            .get(format!("{}/api/tags", base_url))
            .send()
            .await?
            .json()
            .await?;

        let models = response["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    async fn fetch_openrouter_models(
        client: &reqwest::Client,
        base_url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response: serde_json::Value = client
            .get(format!("{}/models", base_url))
            .send()
            .await?
            .json()
            .await?;

        let models = response["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    pub async fn print_models(
        &self,
        base_url: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Available models for {:?}:", self);

        let mut models = self.fetch_models(base_url).await?;
        models.sort();

        if models.is_empty() {
            match self {
                Self::Ollama => {
                    println!("  No models installed. Run 'ollama pull <model>' to install.")
                }
                _ => println!("  No models found."),
            }
            return Ok(());
        }

        let default = self.default_model();
        let display_limit = if matches!(self, Self::Openrouter) {
            50
        } else {
            models.len()
        };

        for model in models.iter().take(display_limit) {
            let marker = if model == default || model.starts_with(default) {
                " (default)"
            } else {
                ""
            };
            println!("  - {}{}", model, marker);
        }

        if models.len() > display_limit {
            println!(
                "  ... and {} more. See https://openrouter.ai/models for full list",
                models.len() - display_limit
            );
        }

        Ok(())
    }
}
