use std::fs;
use std::path::PathBuf;

use crate::config::MergedConfig;

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
