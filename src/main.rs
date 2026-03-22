mod app;
mod biases;
mod ai;
mod ui;
mod config;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[derive(Parser, Debug)]
#[command(name = "cbd")]
#[command(about = "Cognitive Bias Detector — analyse text for cognitive biases")]
#[command(version)]
struct Cli {
    /// Text to analyse directly (skips interactive mode)
    #[arg(short, long)]
    text: Option<String>,

    /// AI provider to use: openai, anthropic, ollama, gemini (overrides config)
    #[arg(short, long)]
    provider: Option<String>,

    /// Output results as JSON (non-interactive)
    #[arg(short, long)]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::Config::load()?;

    // Non-interactive / pipe mode
    if let Some(text) = cli.text {
        let provider_override = cli.provider.or(cfg.ai.as_ref().map(|a| a.provider.clone()));
        let results = biases::engine::analyse(&text);
        let ai_results = if let Some(ref provider) = provider_override {
            let ai_cfg = cfg.ai.clone().unwrap_or_default();
            match ai::analyse_with_ai(&text, provider, &ai_cfg).await {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("AI analysis failed: {e}");
                    None
                }
            }
        } else {
            None
        };

        if cli.json {
            let combined = app::state::CombinedResult { rule_based: results, ai_result: ai_results };
            println!("{}", serde_json::to_string_pretty(&combined)?);
        } else {
            ui::plain::print_results(&text, &results, ai_results.as_ref());
        }
        return Ok(());
    }

    // TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let provider = cli.provider.or_else(|| cfg.ai.as_ref().map(|a| a.provider.clone()));
    let mut app = app::App::new(cfg, provider);
    let res = app.run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}
