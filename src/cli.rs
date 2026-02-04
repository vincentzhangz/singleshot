use crate::provider::Provider;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "singleshot")]
#[command(author, version, about = "Test AI models with a single prompt")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    Chat {
        #[arg(short, long, help = "The prompt to send")]
        prompt: Option<String>,

        #[arg(short, long, help = "File containing the prompt")]
        file: Option<PathBuf>,

        #[arg(short, long, help = "Load config from markdown file")]
        load: Option<PathBuf>,

        #[arg(short, long, help = "Image file to include")]
        image: Option<PathBuf>,

        #[arg(short, long, help = "Video file to include")]
        video: Option<PathBuf>,

        #[arg(short, long, help = "Audio file to include")]
        audio: Option<PathBuf>,

        #[arg(
            short = 'P',
            long,
            value_enum,
            default_value = "ollama",
            help = "AI provider"
        )]
        provider: Provider,

        #[arg(short, long, help = "Model name")]
        model: Option<String>,

        #[arg(short, long, help = "Custom API base URL")]
        base_url: Option<String>,

        #[arg(short, long, help = "System prompt")]
        system: Option<String>,

        #[arg(short, long, default_value = "0.7", help = "Temperature (0.0-2.0)")]
        temperature: f64,

        #[arg(long, help = "Maximum tokens in response")]
        max_tokens: Option<u64>,

        #[arg(long, help = "Maximum tool call turns (default: 10)")]
        max_turns: Option<usize>,

        #[arg(
            long = "mcp",
            help = "MCP server URL(s) - can be specified multiple times (e.g., --mcp http://localhost:8080 --mcp http://localhost:8081)"
        )]
        mcp_server: Vec<String>,

        #[arg(short, long, help = "Show detailed information")]
        detail: bool,

        #[arg(short, long, help = "Generate report file")]
        report: Option<Option<PathBuf>>,
    },

    Models {
        #[arg(short = 'P', long, value_enum, default_value = "ollama")]
        provider: Provider,

        #[arg(short, long)]
        base_url: Option<String>,
    },

    Ping {
        #[arg(short = 'P', long, value_enum, default_value = "ollama")]
        provider: Provider,

        #[arg(short, long)]
        base_url: Option<String>,
    },

    Generate {
        #[arg(short, long, default_value = "example.md")]
        output: PathBuf,
    },
}
