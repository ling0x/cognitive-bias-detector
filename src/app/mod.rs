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

// ─── Unicode-safe cursor helpers ─────────────────────────────────────────────

/// Returns the byte offset of the nth char boundary in `s`.
/// `char_idx` is a char index (not byte offset).
fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

/// Returns the char count of `s`.
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Insert a string at char position `char_idx`.
fn insert_str_at(s: &mut String, char_idx: usize, text: &str) {
    let byte_pos = char_to_byte(s, char_idx);
    s.insert_str(byte_pos, text);
}

/// Insert a single char at char position `char_idx`.
fn insert_char_at(s: &mut String, char_idx: usize, c: char) {
    let byte_pos = char_to_byte(s, char_idx);
    s.insert(byte_pos, c);
}

/// Remove the char at char position `char_idx`.
fn remove_char_at(s: &mut String, char_idx: usize) {
    let byte_pos = char_to_byte(s, char_idx);
    s.remove(byte_pos);
}

// ─────────────────────────────────────────────────────────────────────────────

impl App {
    pub fn new(config: Config, provider: Option<String>) -> Self {
        let ai_enabled = provider.is_some()
            || config.ai.as_ref().map(|a| !a.provider.is_empty()).unwrap_or(false);

        let mut state = AppState::default();
        state.ai_enabled = ai_enabled;

        Self { state, config, provider }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        <B as Backend>::Error: Send + Sync + 'static,
    {
        loop {
            terminal.draw(|f| crate::ui::render(f, &self.state))?;

            if !event::poll(Duration::from_millis(100))? {
                continue;
            }

            let evt = event::read()?;
            let should_quit = self.handle_event(evt).await?;
            if should_quit {
                break;
            }
            terminal.draw(|f| crate::ui::render(f, &self.state))?;
        }
        Ok(())
    }

    /// Returns true if the app should quit.
    async fn handle_event(&mut self, evt: Event) -> Result<bool> {
        match evt {
            // ── Bracketed paste: insert entire pasted string at cursor ──────
            Event::Paste(pasted) if self.state.mode == AppMode::Input => {
                self.handle_paste(&pasted);
            }

            Event::Key(key) if key.kind == KeyEventKind::Press => {
                return self.handle_key(key.code, key.modifiers).await;
            }

            _ => {}
        }
        Ok(false)
    }

    /// Insert pasted text, normalising line endings and filtering control chars.
    fn handle_paste(&mut self, pasted: &str) {
        // Normalise \r\n → \n, strip other control chars except \n and \t
        let cleaned: String = pasted
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .chars()
            .filter(|&c| c == '\n' || c == '\t' || !c.is_control())
            .collect();

        if cleaned.is_empty() {
            return;
        }

        let pos = self.state.cursor_pos;
        insert_str_at(&mut self.state.input_text, pos, &cleaned);
        self.state.cursor_pos += char_len(&cleaned);
        self.state.error_message = None;
    }

    /// Returns true if the app should quit.
    async fn handle_key(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        // Ctrl+C — always quit
        if code == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }

        // Global 'q' to go back (except when typing in input or codex search)
        if code == KeyCode::Char('q')
            && self.state.mode != AppMode::Input
            && !self.state.codex_search_active
        {
            if matches!(
                self.state.mode,
                AppMode::Results | AppMode::BiasDetail | AppMode::CodexBrowser | AppMode::Config
            ) {
                self.reset_to_input();
                return Ok(false);
            }
        }

        match self.state.mode {
            AppMode::Input => self.handle_input_mode(code, mods).await,
            AppMode::Results => self.handle_results_mode(code, mods).await,
            AppMode::BiasDetail => self.handle_detail_mode(code).await,
            AppMode::Config => self.handle_config_mode(code).await,
            AppMode::CodexBrowser => self.handle_codex_mode(code, mods).await,
            AppMode::Analysing => Ok(false),
        }
    }

    fn reset_to_input(&mut self) {
        self.state.mode = AppMode::Input;
        self.state.input_text.clear();
        self.state.cursor_pos = 0;
        self.state.rule_results.clear();
        self.state.ai_result = None;
        self.state.selected_result_idx = 0;
        self.state.scroll_offset = 0;
        self.state.error_message = None;
        self.state.status_message = None;
    }

    async fn handle_input_mode(&mut self, code: KeyCode, mods: KeyModifiers) -> Result<bool> {
        match code {
            KeyCode::Esc => return Ok(true),

            KeyCode::F(1) => {
                self.state.show_help = !self.state.show_help;
            }

            KeyCode::F(2) => {
                self.state.mode = AppMode::CodexBrowser;
                self.state.codex_scroll = 0;
                self.state.codex_search.clear();
                self.state.codex_search_active = false;
            }

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
                        "No AI configured. Add [ai] section to ~/.config/cbd/config.toml"
                            .to_string(),
                    );
                }
            }

            KeyCode::F(4) => {
                self.state.mode = AppMode::Config;
            }

            // F5 or Ctrl+Enter → analyse
            KeyCode::F(5) => {
                self.run_analysis().await?;
            }
            KeyCode::Enter if mods.contains(KeyModifiers::CONTROL) => {
                self.run_analysis().await?;
            }
            KeyCode::Enter if mods.contains(KeyModifiers::ALT) => {
                self.run_analysis().await?;
            }

            // Plain Enter → newline in textarea
            KeyCode::Enter => {
                let pos = self.state.cursor_pos;
                insert_char_at(&mut self.state.input_text, pos, '\n');
                self.state.cursor_pos += 1;
            }

            KeyCode::Char(c) => {
                // Ignore Ctrl+key combinations (except Ctrl+Enter handled above)
                if mods.contains(KeyModifiers::CONTROL) {
                    return Ok(false);
                }
                let pos = self.state.cursor_pos;
                insert_char_at(&mut self.state.input_text, pos, c);
                self.state.cursor_pos += 1;
                self.state.error_message = None;
            }

            KeyCode::Backspace => {
                if self.state.cursor_pos > 0 {
                    let pos = self.state.cursor_pos - 1;
                    remove_char_at(&mut self.state.input_text, pos);
                    self.state.cursor_pos -= 1;
                }
            }

            KeyCode::Delete => {
                let pos = self.state.cursor_pos;
                let len = char_len(&self.state.input_text);
                if pos < len {
                    remove_char_at(&mut self.state.input_text, pos);
                }
            }

            KeyCode::Left => {
                if self.state.cursor_pos > 0 {
                    self.state.cursor_pos -= 1;
                }
            }

            KeyCode::Right => {
                let len = char_len(&self.state.input_text);
                if self.state.cursor_pos < len {
                    self.state.cursor_pos += 1;
                }
            }

            KeyCode::Home => {
                // Move to start of current line
                let text = self.state.input_text.clone();
                let pos = self.state.cursor_pos;
                // Find the char index of the start of the current line
                let before: String = text.chars().take(pos).collect();
                let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                self.state.cursor_pos = line_start;
            }

            KeyCode::End => {
                // Move to end of current line
                let text = self.state.input_text.clone();
                let pos = self.state.cursor_pos;
                let after: String = text.chars().skip(pos).collect();
                let offset = after.find('\n').unwrap_or(after.chars().count());
                self.state.cursor_pos = pos + offset;
            }

            KeyCode::Up => {
                self.move_cursor_vertical(-1);
            }

            KeyCode::Down => {
                self.move_cursor_vertical(1);
            }

            _ => {}
        }
        Ok(false)
    }

    /// Move cursor up (-1) or down (+1) by one line, preserving column.
    fn move_cursor_vertical(&mut self, direction: i32) {
        let text = &self.state.input_text;
        let pos = self.state.cursor_pos;

        // Split into char-indexed lines
        let chars: Vec<char> = text.chars().collect();
        let total = chars.len();

        // Find current line start/end
        let line_start = chars[..pos].iter().rposition(|&c| c == '\n').map(|i| i + 1).unwrap_or(0);
        let col = pos - line_start;

        if direction < 0 {
            // Move up
            if line_start == 0 {
                self.state.cursor_pos = 0;
                return;
            }
            // Previous line ends at line_start - 1
            let prev_line_end = line_start - 1;
            let prev_line_start = chars[..prev_line_end].iter().rposition(|&c| c == '\n').map(|i| i + 1).unwrap_or(0);
            let prev_line_len = prev_line_end - prev_line_start;
            self.state.cursor_pos = prev_line_start + col.min(prev_line_len);
        } else {
            // Move down
            let next_nl = chars[pos..].iter().position(|&c| c == '\n');
            if let Some(offset) = next_nl {
                let next_line_start = pos + offset + 1;
                let next_nl2 = chars[next_line_start..].iter().position(|&c| c == '\n');
                let next_line_len = next_nl2.unwrap_or(total - next_line_start);
                self.state.cursor_pos = next_line_start + col.min(next_line_len);
            } else {
                self.state.cursor_pos = total;
            }
        }
    }

    async fn run_analysis(&mut self) -> Result<()> {
        let text = self.state.input_text.trim().to_string();
        if text.is_empty() {
            self.state.error_message = Some("Please enter some text to analyse.".to_string());
            return Ok(());
        }

        self.state.error_message = None;
        self.state.status_message = Some("Analysing…".to_string());
        self.state.mode = AppMode::Analysing;

        let rule_results = engine::analyse(&text);
        self.state.rule_results = rule_results;
        self.state.last_analysed = Some(chrono::Local::now());

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
        let total = self.state.rule_results.len()
            + self.state.ai_result.as_ref().map(|r| r.detected_biases.len()).unwrap_or(0);

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.state.selected_result_idx > 0 {
                    self.state.selected_result_idx -= 1;
                }
                self.state.scroll_offset = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.state.selected_result_idx + 1 < total {
                    self.state.selected_result_idx += 1;
                }
                self.state.scroll_offset = 0;
            }
            KeyCode::Enter => {
                if total > 0 {
                    self.state.mode = AppMode::BiasDetail;
                    self.state.scroll_offset = 0;
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.reset_to_input();
            }
            KeyCode::Char('e') => {
                if self.config.ai.is_some() {
                    self.state.ai_enabled = true;
                    let text = self.state.input_text.clone();
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
                    self.state.mode = AppMode::Results;
                } else {
                    self.state.error_message =
                        Some("No AI configured. See ~/.config/cbd/config.toml".to_string());
                }
            }
            KeyCode::Char('c') => {
                if !self.state.rule_results.is_empty() {
                    let summary = self
                        .state
                        .rule_results
                        .iter()
                        .map(|r| format!("• {} ({})", r.bias_name, r.confidence_label()))
                        .collect::<Vec<_>>()
                        .join("\n");
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
        let total = self.state.rule_results.len()
            + self.state.ai_result.as_ref().map(|r| r.detected_biases.len()).unwrap_or(0);

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
                if self.state.selected_result_idx > 0 {
                    self.state.selected_result_idx -= 1;
                } else if total > 0 {
                    self.state.selected_result_idx = total - 1;
                }
                self.state.scroll_offset = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
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
                KeyCode::Char(c) if !mods.contains(KeyModifiers::CONTROL) => {
                    self.state.codex_search.push(c);
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
