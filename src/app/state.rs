use crate::ai::AiDetectionResult;
use crate::biases::DetectionResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Main input screen
    Input,
    /// Analysing — spinner shown
    Analysing,
    /// Results view
    Results,
    /// Detailed view for a single bias
    BiasDetail,
    /// Configuration / help screen
    Config,
    /// Codex browser — browse all known biases
    CodexBrowser,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub mode: AppMode,
    pub input_text: String,
    pub cursor_pos: usize,
    pub rule_results: Vec<DetectionResult>,
    pub ai_result: Option<AiDetectionResult>,
    pub selected_result_idx: usize,
    pub scroll_offset: usize,
    pub codex_scroll: usize,
    pub codex_search: String,
    pub codex_search_active: bool,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub ai_loading: bool,
    pub show_help: bool,
    pub ai_enabled: bool,
    /// Analysis timestamp
    pub last_analysed: Option<chrono::DateTime<chrono::Local>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Input,
            input_text: String::new(),
            cursor_pos: 0,
            rule_results: Vec::new(),
            ai_result: None,
            selected_result_idx: 0,
            scroll_offset: 0,
            codex_scroll: 0,
            codex_search: String::new(),
            codex_search_active: false,
            status_message: None,
            error_message: None,
            ai_loading: false,
            show_help: false,
            ai_enabled: false,
            last_analysed: None,
        }
    }
}

/// Used for JSON output in --json mode
#[derive(Debug, Serialize, Deserialize)]
pub struct CombinedResult {
    pub rule_based: Vec<DetectionResult>,
    pub ai_result: Option<AiDetectionResult>,
}
