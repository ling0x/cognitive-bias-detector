# Cognitive Bias Detector

A terminal-based cognitive bias detector written in Rust using [Ratatui](https://ratatui.rs), with optional AI integration. Analyses text for cognitive biases from the full [Cognitive Bias Codex](https://upload.wikimedia.org/wikipedia/commons/6/65/Cognitive_bias_codex_en.svg).

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-TUI-cyan?style=for-the-badge)

---

## Features

- **180+ cognitive biases** from all 10 categories of the Cognitive Bias Codex
- **Rule-based detection engine** — fast, offline, keyword + phrase pattern matching
- **AI-powered analysis** — integrates with OpenAI, Anthropic Claude, Google Gemini, or Ollama (local)
- **Full TUI** — interactive Ratatui interface with keyboard navigation
- **Codex browser** — browse and search the full cognitive bias codex in-terminal
- **Detailed view** — description, evidence, confidence bars, mitigation tips
- **Non-interactive mode** — pipe text in, get JSON or plain-text results out
- **Clipboard export** — copy results to clipboard (wl-copy / xclip)

---

## Installation

```bash
cargo install --path .
```

Or build a release binary:

```bash
cargo build --release
# Binary at: target/release/cbd
```

---

## Usage

### Interactive TUI

```bash
cbd
```

### Analyse text directly (non-interactive)

```bash
cbd --text "I've already put in 3 years. Can't quit now — too much invested."
```

### Use AI provider

```bash
cbd --text "I knew this would happen all along." --provider openai
```

### JSON output

```bash
cbd --text "Everyone agrees this is the right approach." --json
```

---

## Keyboard Shortcuts

| Key               | Action                              |
|-------------------|-------------------------------------|
| `F5` / `Ctrl+Enter` | Analyse the entered text           |
| `F2`              | Browse the full Cognitive Bias Codex |
| `F3`              | Toggle AI analysis on/off           |
| `F4`              | Config / help screen                |
| `↑ ↓` / `j k`    | Navigate results                    |
| `Enter`           | Open detailed bias view             |
| `← →` / `h l`    | Navigate between bias details       |
| `e`               | Run/re-run AI analysis from results |
| `c`               | Copy results to clipboard           |
| `q` / `Esc`       | Go back / return to input           |
| `Ctrl+C`          | Force quit                          |
| `/` (in codex)    | Search the codex                    |

---

## AI Configuration

Create `~/.config/cbd/config.toml`:

```toml
[ai]
# Provider: "openai" | "anthropic" | "gemini" | "ollama"
provider = "openai"
api_key = "sk-..."
model = "gpt-4o-mini"        # optional
max_tokens = 1024

[ui]
show_examples = true
```

### Ollama (local, no API key)

```toml
[ai]
provider = "ollama"
model = "llama3.2"
base_url = "http://localhost:11434"  # optional
```

---

## Bias Categories

All 10 categories from the Cognitive Bias Codex are covered:

| Category | Description |
|---|---|
| Memory & Recall | Distortions in how we remember events |
| Meaning-Making | Finding patterns and purpose where none exists |
| Action & Inertia | Biases affecting whether we act or stay put |
| Recency & Salience | Overweighting recent or vivid information |
| Belief & Confirmation | Protecting and reinforcing existing beliefs |
| Social & Group Dynamics | Biases arising from group membership |
| Probability & Statistics | Errors in reasoning about chance and frequency |
| Self-Perception | Distorted views of oneself |
| Causal Attribution | Errors in assigning cause and blame |
| Decision Making | Biases under uncertainty and choice |

---

## Architecture

```
src/
├── main.rs              # Entry point, CLI parsing
├── app/
│   ├── mod.rs           # App struct, event loop, key handling
│   └── state.rs         # AppState, AppMode, CombinedResult
├── biases/
│   ├── codex.rs         # All 180+ biases with metadata
│   ├── engine.rs        # Rule-based detection engine
│   └── patterns.rs      # Keyword & phrase patterns
├── ai/
│   ├── mod.rs           # Provider dispatcher
│   ├── prompt.rs        # System/user prompt builders
│   ├── openai.rs        # OpenAI API integration
│   ├── anthropic.rs     # Anthropic API integration
│   ├── gemini.rs        # Google Gemini API integration
│   └── ollama.rs        # Ollama local API integration
├── ui/
│   ├── mod.rs           # Root renderer, header, statusbar
│   ├── input.rs         # Text input screen
│   ├── results.rs       # Results list + preview
│   ├── detail.rs        # Full bias detail view
│   ├── codex.rs         # Codex browser
│   ├── config.rs        # Config / help screen
│   ├── widgets.rs       # Reusable widgets (confidence bar, badges)
│   └── plain.rs         # Non-TUI plain-text output
└── config/
    └── mod.rs           # Config loading/saving
```

---

## Reference

- [Cognitive Bias Codex](https://upload.wikimedia.org/wikipedia/commons/6/65/Cognitive_bias_codex_en.svg) — by Buster Benson and John Manoogian III
- [List of Cognitive Biases — Wikipedia](https://en.wikipedia.org/wiki/List_of_cognitive_biases)
- Inspired by [CDFire's Cognitive Bias Detector](https://cdfire.github.io/CognitiveBiasDetector/)

---

## License

MIT
