mod chat;
mod cli;
mod config;
mod mcp;
mod provider;
mod report;
mod ui;

use base64::Engine;
use chat::{ChatConfig, send_prompt};
use clap::Parser;
use cli::{Cli, Commands};
use config::{LoadedConfig, MergedConfig, TEMPLATE};
use mcp::McpSession;
use provider::Provider;
use report::Report;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::time::Instant;
use ui::{BoxPrinter, completed, newline, status, status_done, status_done_detail};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("\nError: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Chat {
            prompt,
            file,
            load,
            image,
            video,
            audio,
            provider,
            model,
            base_url,
            system,
            temperature,
            max_tokens,
            max_turns,
            mcp_server,
            detail,
            report,
        } => {
            handle_chat(
                prompt,
                file,
                load,
                image,
                video,
                audio,
                provider,
                model,
                base_url,
                system,
                temperature,
                max_tokens,
                max_turns,
                mcp_server,
                detail,
                report,
            )
            .await?;
        }

        Commands::Models { provider, base_url } => {
            provider.setup_base_url(base_url.as_deref());
            provider.print_models(base_url.as_deref()).await?;
        }

        Commands::Ping { provider, base_url } => {
            handle_ping(&provider, base_url.as_deref()).await?;
        }

        Commands::Generate { output } => {
            fs::write(&output, TEMPLATE)?;
            println!("Generated example config: {}", output.display());
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_chat(
    prompt: Option<String>,
    file: Option<PathBuf>,
    load: Option<PathBuf>,
    image: Option<PathBuf>,
    video: Option<PathBuf>,
    audio: Option<PathBuf>,
    cli_provider: Provider,
    cli_model: Option<String>,
    cli_base_url: Option<String>,
    cli_system: Option<String>,
    cli_temperature: f64,
    cli_max_tokens: Option<u64>,
    cli_max_turns: Option<usize>,
    mcp_servers: Vec<String>,
    detail: bool,
    report: Option<Option<PathBuf>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load.as_ref().map(LoadedConfig::from_file).transpose()?;
    let config = MergedConfig::new(
        loaded.as_ref(),
        &cli_provider,
        cli_model.as_deref(),
        cli_base_url.as_deref(),
        cli_system.as_deref(),
        cli_temperature,
        cli_max_tokens,
        cli_max_turns,
        &mcp_servers,
    );

    let prompt_text = resolve_prompt(prompt, file, config.prompt.clone())?;
    let system = config.system.map(String::from);

    let image_path = image.or_else(|| config.image.clone());
    let video_path = video.or_else(|| config.video.clone());
    let audio_path = audio.or_else(|| config.audio.clone());

    let image_data = image_path.as_ref().map(encode_file_to_base64).transpose()?;
    let video_data = video_path.as_ref().map(encode_file_to_base64).transpose()?;
    let audio_data = audio_path.as_ref().map(encode_file_to_base64).transpose()?;

    let has_media = image_data.is_some() || video_data.is_some() || audio_data.is_some();

    let mut mcp_sessions: Vec<McpSession> = Vec::new();
    for server_url in &config.mcp_servers {
        status(&format!("Connecting to MCP server: {}", server_url));
        let session = McpSession::connect(server_url).await?;
        let tool_names = session.tool_names();
        status_done_detail(
            "Connected to MCP server",
            &format!("{} tools: {}", session.tool_count(), tool_names.join(", ")),
        );
        mcp_sessions.push(session);
    }

    config.provider.setup_base_url(config.base_url);

    if detail {
        print_config_box(&config, &prompt_text, &system, has_media, &mcp_sessions);
    }

    status("Sending request...");
    let start = Instant::now();

    let chat_config = ChatConfig::new(config.model_name())
        .system(system.clone())
        .temperature(config.temperature)
        .max_tokens(config.max_tokens)
        .max_turns(config.max_turns)
        .image(image_data)
        .video(video_data)
        .audio(audio_data)
        .base_url(config.base_url.map(String::from))
        .mcp_sessions(mcp_sessions);

    let response = send_prompt(config.provider, &prompt_text, chat_config).await?;
    let elapsed = start.elapsed();

    if detail {
        status_done_detail(
            "Received response",
            &format!("{:.2}s", elapsed.as_secs_f64()),
        );
    } else {
        status_done("Received response");
    }

    newline();
    println!("{}", response);

    if let Some(report_path) = report {
        let path = report_path.unwrap_or_else(|| PathBuf::from("report.md"));
        let r = Report::generate(&config, &prompt_text, system.as_deref(), &response, elapsed);
        r.save(&path)?;
        newline();
        completed(&format!("Report saved: {}", path.display()));
    }

    Ok(())
}

fn resolve_prompt(
    arg: Option<String>,
    file: Option<PathBuf>,
    loaded: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(text) = arg {
        return Ok(text);
    }

    if let Some(path) = file {
        return Ok(fs::read_to_string(path)?);
    }

    if let Some(text) = loaded {
        return Ok(text);
    }

    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if !buffer.is_empty() {
            return Ok(buffer);
        }
    }

    Err("No prompt provided. Use --prompt, --file, --load, or pipe from stdin.".into())
}

fn encode_file_to_base64(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&data))
}

fn print_config_box(
    config: &MergedConfig,
    prompt: &str,
    system: &Option<String>,
    has_media: bool,
    mcp_sessions: &[McpSession],
) {
    let printer = BoxPrinter::new(60);

    printer.print_top();
    printer.print_line(&format!("Provider:    {:?}", config.provider));
    printer.print_line(&format!("Model:       {}", config.model_name()));
    printer.print_line(&format!("Temperature: {}", config.temperature));

    if let Some(tokens) = config.max_tokens {
        printer.print_line(&format!("Max Tokens:  {}", tokens));
    }
    printer.print_line(&format!("Max Turns:   {}", config.max_turns));
    if has_media {
        printer.print_line("Media:       attached");
    }
    if !mcp_sessions.is_empty() {
        let total_tools: usize = mcp_sessions.iter().map(|s| s.tool_count()).sum();
        printer.print_line(&format!(
            "MCP Servers: {} ({} tools)",
            mcp_sessions.len(),
            total_tools
        ));
    }

    printer.print_separator();
    printer.print_line("Prompt:");

    for line in prompt.lines().take(5) {
        let truncated = if line.len() > 56 {
            format!("{}...", &line[..53])
        } else {
            line.to_string()
        };
        printer.print_line(&format!("  {}", truncated));
    }

    if prompt.lines().count() > 5 {
        printer.print_line(&format!(
            "  ... ({} more lines)",
            prompt.lines().count() - 5
        ));
    }

    if let Some(sys) = system {
        printer.print_separator();
        printer.print_line("System:");
        let truncated = if sys.len() > 54 {
            format!("{}...", &sys[..51])
        } else {
            sys.clone()
        };
        printer.print_line(&format!("  {}", truncated));
    }

    printer.print_bottom();
    newline();
}

async fn handle_ping(
    provider: &Provider,
    base_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    provider.setup_base_url(base_url);

    status(&format!("Pinging {:?}...", provider));
    let start = Instant::now();

    let chat_config = ChatConfig::new(provider.ping_model())
        .max_tokens(Some(10))
        .base_url(base_url.map(String::from));
    let _ = send_prompt(provider, "Say 'pong'", chat_config).await?;

    let elapsed = start.elapsed();
    status_done_detail("Ping successful", &format!("{:.0}ms", elapsed.as_millis()));

    Ok(())
}
