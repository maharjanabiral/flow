use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hotkey: String,
    #[serde(default = "default_model_path")]
    pub model_path: String,
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
}

fn default_model_path() -> String {
    "models/ggml-base.en.bin".to_string()
}

fn default_llm_model() -> String {
    "llama3.2:1b".to_string()
}

impl Config {
    /// Loads config from `config.toml` in the current directory.
    /// If the file doesn't exist, creates one with sensible defaults
    /// so the app is usable on first run without manual setup.
    pub fn load_or_create(path: &str) -> Self {
        if !Path::new(path).exists() {
            println!("No config found at {path}, creating one with defaults...");
            let default_contents = r#"# Push-to-talk hotkey.
# Supported keys: F1-F12, ScrollLock, Pause, Insert
# Supported modifiers: Shift, Ctrl, Alt (can combine, e.g. "Ctrl+Alt+F9")
hotkey = "F9"

# Path to the whisper.cpp GGML model file.
model_path = "models/ggml-base.en.bin"
"#;
            std::fs::write(path, default_contents).expect("failed to write default config.toml");
        }

        let contents =
            std::fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read {path}"));

        toml::from_str(&contents).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
    }
}
