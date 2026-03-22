pub mod state;

use crate::ai;
use crate::biases::engine;
use crate::config::Config;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use state::{AppMode, AppState};
use std::time::Duration;

pub struct App {
    pub state: AppState,
    pub config: Config,
    pub provider: Option<String>,
}

impl App {
    pub fn new(config: Config, provider: Option<String>) -> Self {
        let ai_enabled = provider.is_some()
            || config.ai.as_ref().map(|a| !a.provider.is_empty()).unwrap_or(false);

        let mut state = AppState::default();
        state.ai_enabled = ai_enabled;

        Self { state, config, provider }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| crate::ui::render(f, &self.state))?;

            if !event::poll(Duration::from_millis(100))? {
                continue;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if self.handle_key(key.code, key.modifiers).await? {
                    break;
                }
                // re-draw after each key
                terminal.draw(|f| crate::ui::render(f, &self.state))?;
            }
        }
        Ok(())
    }

    /// Returns true if the app should quit
    async fn handle_key(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        // Global quit
        if code == KeyCode::Char('q') && self.state.mode != AppMode::Input && !self.state.codex_search_active {
            if self.state.mode == AppMode::Results
                || self.state.mode == AppMode::BiasDetail
                || self.state.mode == AppMode::CodexBrowser
                || self.state.mode == AppMode::Config
            {
                self.state.mode = AppMode::Input;
                self.state.input_text.clear();
                self.state.cursor_pos = 0;
                self.state.rule_results.clear();
                self.state.ai_result = None;
                self.state.selected_result_idx = 0;
                self.state.scroll_offset = 0;
                self.state.error_message = None;
                self.state.status_message = None;
                return Ok(false);
            }
        }

        // Ctrl+C — always quit
        if code == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }

        match self.state.mode {
            AppMode::Input => self.handle_input_mode(code, mods).await,
            AppMode::Results => self.handle_results_mode(code, mods).await,
            AppMode::BiasDetail => self.handle_detail_mode(code).await,
            AppMode::Config => self.handle_config_mode(code).await,
            AppMode::CodexBrowser => self.handle_codex_mode(code, mods).await,
            AppMode::Analysing => Ok(false), // ignore input while analysing
        }
    }

    async fn handle_input_mode(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        match code {
            // Ctrl+Q or Escape in input — quit
            KeyCode::Esc => return Ok(true),

            // F1 / ? — toggle help
            KeyCode::F(1) => {
                self.state.show_help = !self.state.show_help;
            }

            // F2 — open codex browser
            KeyCode::F(2) => {
                self.state.mode = AppMode::CodexBrowser;
                self.state.codex_scroll = 0;
                self.state.codex_search.clear();
                self.state.codex_search_active = false;
            }

            // F3 — toggle AI
            KeyCode::F(3) => {
                if self.config.ai.is_some() {
                    self.state.ai_enabled = !self.state.ai_enabled;
                    self.state.status_message = Some(if self.state.ai_enabled {
                        "AI analysis enabled".to_string()
                    } else {
                        "AI analysis disabled".to_string()
                    });
                } else {
                    self.state.error_message = Some(
                        "No AI configured. Add [ai] section to ~/.config/cbd/config.toml".to_string(),
                    );
                }
            }

            // F4 — config screen
            KeyCode::F(4) => {
                self.state.mode = AppMode::Config;
            }

            // Enter — analyse
            KeyCode::Enter if mods.contains(KeyModifiers::ALT) || mods.contains(KeyModifiers::CONTROL) => {
                self.run_analysis().await?;
            }
            KeyCode::F(5) => {
                self.run_analysis().await?;
            }

            // Regular enter = newline in textarea
            KeyCode::Enter => {
                let pos = self.state.cursor_pos;
                self.state.input_text.insert(pos, '\n');
                self.state.cursor_pos += 1;
            }

            KeyCode::Char(c) => {
                let pos = self.state.cursor_pos;
                self.state.input_text.insert(pos, c);
                self.state.cursor_pos += 1;
                self.state.error_message = None;
            }

            KeyCode::Backspace => {
                if self.state.cursor_pos > 0 {
                    let pos = self.state.cursor_pos - 1;
                    self.state.input_text.remove(pos);
                    self.state.cursor_pos -= 1;
                }
            }

            KeyCode::Delete => {
                let pos = self.state.cursor_pos;
                if pos < self.state.input_text.len() {
                    self.state.input_text.remove(pos);
                }
            }

            KeyCode::Left => {
                if self.state.cursor_pos > 0 {
                    self.state.cursor_pos -= 1;
                }
            }

            KeyCode::Right => {
                let len = self.state.input_text.len();
                if self.state.cursor_pos < len {
                    self.state.cursor_pos += 1;
                }
            }

            KeyCode::Home => {
                self.state.cursor_pos = 0;
            }

            KeyCode::End => {
                self.state.cursor_pos = self.state.input_text.len();
            }

            KeyCode::Up => {
                // Move cursor up a line
                let text = &self.state.input_text;
                let pos = self.state.cursor_pos;
                if let Some(prev_nl) = text[..pos].rfind('\n') {
                    let line_start = text[..prev_nl].rfind('\n').map(|p| p + 1).unwrap_or(0);
                    let col = pos - (prev_nl + 1);
                    let prev_line_len = prev_nl - line_start;
                    self.state.cursor_pos = line_start + col.min(prev_line_len);
                } else {
                    self.state.cursor_pos = 0;
                }
            }

            KeyCode::Down => {
                let text = self.state.input_text.clone();
                let pos = self.state.cursor_pos;
                let current_line_start = text[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let col = pos - current_line_start;
                if let Some(next_nl_offset) = text[pos..].find('\n') {
                    let next_line_start = pos + next_nl_offset + 1;
                    let next_line_end = text[next_line_start..]
                        .find('\n')
                        .map(|p| next_line_start + p)
                        .unwrap_or(text.len());
                    let next_line_len = next_line_end - next_line_start;
                    self.state.cursor_pos = next_line_start + col.min(next_line_len);
                } else {
                    self.state.cursor_pos = text.len();
                }
            }

            _ => {}
        }
        Ok(false)
    }

    async fn run_analysis(&mut self) -> Result<()> {
        let text = self.state.input_text.trim().to_string();
        if text.is_empty() {
            self.state.error_message = Some("Please enter some text to analyse.".to_string());
            return Ok(());
        }

        self.state.error_message = None;
        self.state.status_message = Some("Analysing...".to_string());
        self.state.mode = AppMode::Analysing;

        // Run rule-based analysis (synchronous, fast)
        let rule_results = engine::analyse(&text);
        self.state.rule_results = rule_results;
        self.state.last_analysed = Some(chrono::Local::now());

        // Run AI analysis if enabled
        if self.state.ai_enabled {
            let provider = self
                .provider
                .clone()
                .or_else(|| self.config.ai.as_ref().map(|a| a.provider.clone()))
                .unwrap_or_default();

            let ai_cfg = self.config.ai.clone().unwrap_or_default();
            self.state.ai_loading = true;

            match ai::analyse_with_ai(&text, &provider, &ai_cfg).await {
                Ok(result) => {
                    self.state.ai_result = Some(result);
                    self.state.status_message = Some("AI analysis complete.".to_string());
                }
                Err(e) => {
                    self.state.error_message = Some(format!("AI error: {e}"));
                    self.state.ai_result = None;
                }
            }
            self.state.ai_loading = false;
        } else {
            self.state.ai_result = None;
            self.state.status_message = None;
        }

        self.state.selected_result_idx = 0;
        self.state.scroll_offset = 0;
        self.state.mode = AppMode::Results;

        Ok(())
    }

    async fn handle_results_mode(&mut self, code: KeyCode, _mods: KeyModifiers) -> Result<bool> {
        let total = self.state.rule_results.len();
        let ai_count = self
            .state
            .ai_result
            .as_ref()
            .map(|r| r.detected_biases.len())
            .unwrap_or(0);
        let combined_total = total + ai_count;

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.selected_result_idx > 0 {
                    self.state.selected_result_idx -= 1;
                }
                self.state.scroll_offset = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.state.selected_result_idx + 1 < combined_total {
                    self.state.selected_result_idx += 1;
                }
                self.state.scroll_offset = 0;
            }
            KeyCode::Enter => {
                if combined_total > 0 {
                    self.state.mode = AppMode::BiasDetail;
                    self.state.scroll_offset = 0;
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state.mode = AppMode::Input;
                self.state.input_text.clear();
                self.state.cursor_pos = 0;
                self.state.rule_results.clear();
                self.state.ai_result = None;
                self.state.selected_result_idx = 0;
            }
            KeyCode::Char('e') => {
                // Re-analyse with AI
                if self.config.ai.is_some() {
                    self.state.ai_enabled = true;
                    let text = self.state.input_text.clone();
                    // Re-run full analysis
                    let mode_backup = AppMode::Results;
                    self.state.mode = AppMode::Analysing;
                    let provider = self
                        .provider
                        .clone()
                        .or_else(|| self.config.ai.as_ref().map(|a| a.provider.clone()))
                        .unwrap_or_default();
                    let ai_cfg = self.config.ai.clone().unwrap_or_default();
                    match ai::analyse_with_ai(&text, &provider, &ai_cfg).await {
                        Ok(result) => {
                            self.state.ai_result = Some(result);
                            self.state.status_message = Some("AI analysis complete.".to_string());
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("AI error: {e}"));
                        }
                    }
                    let _ = mode_backup;
                    self.state.mode = AppMode::Results;
                } else {
                    self.state.error_message =
                        Some("No AI configured. See ~/.config/cbd/config.toml".to_string());
                }
            }
            KeyCode::Char('c') => {
                // Copy results summary to clipboard via xclip/wl-copy (best-effort)
                if !self.state.rule_results.is_empty() {
                    let summary = self
                        .state
                        .rule_results
                        .iter()
                        .map(|r| format!("• {} ({})", r.bias_name, r.confidence_label()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    // Try wl-copy (Wayland) then xclip (X11)
                    let _ = std::process::Command::new("wl-copy").arg(&summary).status();
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("echo '{}' | xclip -sel clip", summary))
                        .status();
                    self.state.status_message = Some("Results copied to clipboard.".to_string());
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_detail_mode(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.state.mode = AppMode::Results;
                self.state.scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.scroll_offset > 0 {
                    self.state.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.scroll_offset += 1;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let total = self.state.rule_results.len()
                    + self
                        .state
                        .ai_result
                        .as_ref()
                        .map(|r| r.detected_biases.len())
                        .unwrap_or(0);
                if self.state.selected_result_idx > 0 {
                    self.state.selected_result_idx -= 1;
                    self.state.scroll_offset = 0;
                } else if total > 0 {
                    self.state.selected_result_idx = total - 1;
                    self.state.scroll_offset = 0;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let total = self.state.rule_results.len()
                    + self
                        .state
                        .ai_result
                        .as_ref()
                        .map(|r| r.detected_biases.len())
                        .unwrap_or(0);
                if total > 0 {
                    self.state.selected_result_idx = (self.state.selected_result_idx + 1) % total;
                    self.state.scroll_offset = 0;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_config_mode(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.state.mode = AppMode::Input;
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_codex_mode(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        if self.state.codex_search_active {
            match code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.state.codex_search_active = false;
                }
                KeyCode::Backspace => {
                    self.state.codex_search.pop();
                }
                KeyCode::Char(c) => {
                    if !mods.contains(KeyModifiers::CONTROL) {
                        self.state.codex_search.push(c);
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.state.mode = AppMode::Input;
                self.state.codex_search.clear();
                self.state.codex_search_active = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.codex_scroll > 0 {
                    self.state.codex_scroll -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.codex_scroll += 1;
            }
            KeyCode::Char('/') => {
                self.state.codex_search_active = true;
            }
            KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => {
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }
}
