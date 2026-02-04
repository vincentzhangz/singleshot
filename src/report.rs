use crate::config::MergedConfig;
use std::fs;
use std::path::PathBuf;

pub struct Report {
    content: String,
}

impl Report {
    pub fn new() -> Self {
        Self {
            content: String::with_capacity(4096),
        }
    }

    pub fn generate(
        config: &MergedConfig,
        prompt_text: &str,
        system: Option<&str>,
        response: &str,
        elapsed: std::time::Duration,
    ) -> Self {
        let mut report = Self::new();
        let model_name = config.model_name();
        let elapsed_secs = elapsed.as_secs_f64();

        report.add_header();
        report.add_configuration(config, &model_name);
        report.add_system_prompt(system);
        report.add_prompt(prompt_text);
        report.add_media_files(&config.image, &config.video, &config.audio);
        report.add_response(response);
        report.add_performance(prompt_text, response, elapsed_secs);

        report
    }

    fn add_header(&mut self) {
        self.content.push_str("# Single Shot Test Report\n\n");
        self.content.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
    }

    fn add_configuration(&mut self, config: &MergedConfig, model_name: &str) {
        self.content.push_str("## Configuration\n\n");
        self.content
            .push_str(&format!("- **Provider:** {:?}\n", config.provider));
        self.content
            .push_str(&format!("- **Model:** {}\n", model_name));

        if let Some(url) = &config.base_url {
            self.content.push_str(&format!("- **Base URL:** {}\n", url));
        }

        self.content
            .push_str(&format!("- **Temperature:** {}\n", config.temperature));

        if let Some(tokens) = config.max_tokens {
            self.content
                .push_str(&format!("- **Max Tokens:** {}\n", tokens));
        }
    }

    fn add_system_prompt(&mut self, system: Option<&str>) {
        if let Some(sys) = system {
            self.content.push_str("\n## System Prompt\n\n```\n");
            self.content.push_str(sys);
            self.content.push_str("\n```\n");
        }
    }

    fn add_prompt(&mut self, prompt: &str) {
        self.content.push_str("\n## Prompt\n\n```\n");
        self.content.push_str(prompt);
        self.content.push_str("\n```\n");
    }

    fn add_media_files(
        &mut self,
        image: &Option<PathBuf>,
        video: &Option<PathBuf>,
        audio: &Option<PathBuf>,
    ) {
        if image.is_some() || video.is_some() || audio.is_some() {
            self.content.push_str("\n## Media Files\n\n");

            if let Some(img) = image {
                self.content.push_str(&format!("- **Image:** {:?}\n", img));
            }
            if let Some(vid) = video {
                self.content.push_str(&format!("- **Video:** {:?}\n", vid));
            }
            if let Some(aud) = audio {
                self.content.push_str(&format!("- **Audio:** {:?}\n", aud));
            }
        }
    }

    fn add_response(&mut self, response: &str) {
        self.content.push_str("\n## Response\n\n```\n");
        self.content.push_str(response);
        self.content.push_str("\n```\n");
    }

    fn add_performance(&mut self, prompt: &str, response: &str, elapsed_secs: f64) {
        self.content.push_str("\n## Performance\n\n");
        self.content
            .push_str(&format!("- **Duration:** {:.3}s\n", elapsed_secs));
        self.content
            .push_str(&format!("- **Prompt Length:** {} chars\n", prompt.len()));
        self.content.push_str(&format!(
            "- **Response Length:** {} chars\n",
            response.len()
        ));
        self.content.push_str(&format!(
            "- **Estimated Prompt Tokens:** ~{}\n",
            prompt.split_whitespace().count()
        ));
        self.content.push_str(&format!(
            "- **Estimated Response Tokens:** ~{}\n",
            response.split_whitespace().count()
        ));
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(path, &self.content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Provider;
    use std::time::Duration;

    fn create_test_config<'a>(provider: &'a Provider) -> MergedConfig<'a> {
        MergedConfig::new(
            None,
            provider,
            Some("test-model"),
            Some("https://api.test.com"),
            Some("System prompt"),
            0.7,
            Some(1000),
            None,
            &[],
        )
    }

    #[test]
    fn test_report_new() {
        let report = Report::new();
        assert!(report.content.is_empty());
    }

    #[test]
    fn test_report_generate_contains_header() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(
            &config,
            "Test prompt",
            Some("System"),
            "Response text",
            Duration::from_secs(1),
        );

        assert!(report.content.contains("# Single Shot Test Report"));
        assert!(report.content.contains("**Generated:**"));
    }

    #[test]
    fn test_report_generate_contains_configuration() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(
            &config,
            "Test prompt",
            None,
            "Response",
            Duration::from_secs(1),
        );

        assert!(report.content.contains("## Configuration"));
        assert!(report.content.contains("**Provider:**"));
        assert!(report.content.contains("**Model:**"));
        assert!(report.content.contains("**Temperature:**"));
    }

    #[test]
    fn test_report_generate_contains_prompt() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(
            &config,
            "My test prompt",
            None,
            "Response",
            Duration::from_secs(1),
        );

        assert!(report.content.contains("## Prompt"));
        assert!(report.content.contains("My test prompt"));
    }

    #[test]
    fn test_report_generate_contains_system_prompt() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(
            &config,
            "Prompt",
            Some("Be helpful"),
            "Response",
            Duration::from_secs(1),
        );

        assert!(report.content.contains("## System Prompt"));
        assert!(report.content.contains("Be helpful"));
    }

    #[test]
    fn test_report_generate_no_system_prompt_when_none() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(!report.content.contains("## System Prompt"));
    }

    #[test]
    fn test_report_generate_contains_response() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(
            &config,
            "Prompt",
            None,
            "This is the AI response",
            Duration::from_secs(1),
        );

        assert!(report.content.contains("## Response"));
        assert!(report.content.contains("This is the AI response"));
    }

    #[test]
    fn test_report_generate_contains_performance() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(
            &config,
            "Test prompt",
            None,
            "Response text",
            Duration::from_millis(1500),
        );

        assert!(report.content.contains("## Performance"));
        assert!(report.content.contains("**Duration:**"));
        assert!(report.content.contains("**Prompt Length:**"));
        assert!(report.content.contains("**Response Length:**"));
        assert!(report.content.contains("**Estimated Prompt Tokens:**"));
        assert!(report.content.contains("**Estimated Response Tokens:**"));
    }

    #[test]
    fn test_report_generate_performance_values() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let prompt = "word1 word2 word3";
        let response = "response1 response2";
        let report = Report::generate(&config, prompt, None, response, Duration::from_secs(2));

        assert!(report.content.contains(&format!("{} chars", prompt.len())));
        assert!(
            report
                .content
                .contains(&format!("{} chars", response.len()))
        );
        assert!(report.content.contains("~3"));
        assert!(report.content.contains("~2"));
    }

    #[test]
    fn test_report_generate_with_media_files() {
        let mut loaded = crate::config::LoadedConfig::default();
        loaded.image = Some(PathBuf::from("/path/to/image.jpg"));
        loaded.video = Some(PathBuf::from("/path/to/video.mp4"));
        loaded.audio = Some(PathBuf::from("/path/to/audio.mp3"));

        let provider = Provider::Openai;
        let config = MergedConfig::new(
            Some(&loaded),
            &provider,
            Some("test-model"),
            None,
            None,
            0.7,
            None,
            None,
            &[],
        );

        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(report.content.contains("## Media Files"));
        assert!(report.content.contains("**Image:**"));
        assert!(report.content.contains("**Video:**"));
        assert!(report.content.contains("**Audio:**"));
    }

    #[test]
    fn test_report_generate_no_media_section_when_none() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(!report.content.contains("## Media Files"));
    }

    #[test]
    fn test_report_generate_with_base_url() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(report.content.contains("**Base URL:**"));
        assert!(report.content.contains("https://api.test.com"));
    }

    #[test]
    fn test_report_generate_with_max_tokens() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(report.content.contains("**Max Tokens:**"));
        assert!(report.content.contains("1000"));
    }

    #[test]
    fn test_report_save() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_report.md");

        report.save(&temp_file).unwrap();

        let content = fs::read_to_string(&temp_file).unwrap();
        assert!(content.contains("# Single Shot Test Report"));

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_report_model_name_uses_config() {
        let provider = Provider::Openai;
        let config = create_test_config(&provider);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(report.content.contains("test-model"));
    }

    #[test]
    fn test_report_model_name_fallback_to_default() {
        let provider = Provider::Openai;
        let config = MergedConfig::new(None, &provider, None, None, None, 0.7, None, None, &[]);
        let report = Report::generate(&config, "Prompt", None, "Response", Duration::from_secs(1));

        assert!(report.content.contains("gpt-4o"));
    }
}
