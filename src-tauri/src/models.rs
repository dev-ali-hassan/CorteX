use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RewriteMode {
    FixGrammar,
    Professional,
    Friendly,
    Shorter,
    Translate,
    Summarize,
    Confident,
    Simplify,
}

impl RewriteMode {
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::FixGrammar => "fixGrammar",
            Self::Professional => "professional",
            Self::Friendly => "friendly",
            Self::Shorter => "shorter",
            Self::Translate => "translate",
            Self::Summarize => "summarize",
            Self::Confident => "confident",
            Self::Simplify => "simplify",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::FixGrammar => "Fix grammar",
            Self::Professional => "Professional",
            Self::Friendly => "Friendly",
            Self::Shorter => "Shorter",
            Self::Translate => "Translate",
            Self::Summarize => "Summarize",
            Self::Confident => "Confident",
            Self::Simplify => "Simplify",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Offline,
    Openai,
    Openrouter,
    Gemini,
    Anthropic,
    Ollama,
}

impl ProviderId {
    pub fn as_id(&self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::Openai => "openai",
            Self::Openrouter => "openrouter",
            Self::Gemini => "gemini",
            Self::Anthropic => "anthropic",
            Self::Ollama => "ollama",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSettings {
    pub provider: ProviderId,
    pub model: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub use_offline_fallback: bool,
    #[serde(default)]
    pub custom_prompt: String,
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            provider: ProviderId::Offline,
            model: "local-cleanup".to_string(),
            api_key: None,
            endpoint: None,
            temperature: 0.35,
            max_tokens: 700,
            use_offline_fallback: true,
            custom_prompt: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub accent_color: String,
    pub launch_at_startup: bool,
    pub auto_replace: bool,
    pub auto_copy: bool,
    pub default_language: String,
    #[serde(default)]
    pub custom_prompt: String,
    pub global_shortcut: String,
    pub grammar_shortcut: String,
    pub professional_shortcut: String,
    #[serde(default = "default_friendly_shortcut")]
    pub friendly_shortcut: String,
    #[serde(default = "default_shorter_shortcut")]
    pub shorter_shortcut: String,
    #[serde(default = "default_translate_shortcut")]
    pub translate_shortcut: String,
    #[serde(default = "default_summarize_shortcut")]
    pub summarize_shortcut: String,
    #[serde(default = "default_confident_shortcut")]
    pub confident_shortcut: String,
    #[serde(default = "default_simplify_shortcut")]
    pub simplify_shortcut: String,
    pub provider: ProviderSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            accent_color: "#8b5cf6".to_string(),
            launch_at_startup: false,
            auto_replace: true,
            auto_copy: false,
            default_language: "English".to_string(),
            custom_prompt: String::new(),
            global_shortcut: "Ctrl + Alt + Z".to_string(),
            grammar_shortcut: "Ctrl + 1".to_string(),
            professional_shortcut: "Ctrl + 2".to_string(),
            friendly_shortcut: default_friendly_shortcut(),
            shorter_shortcut: default_shorter_shortcut(),
            translate_shortcut: default_translate_shortcut(),
            summarize_shortcut: default_summarize_shortcut(),
            confident_shortcut: default_confident_shortcut(),
            simplify_shortcut: default_simplify_shortcut(),
            provider: ProviderSettings::default(),
        }
    }
}

fn default_friendly_shortcut() -> String {
    "Ctrl + 3".to_string()
}

fn default_shorter_shortcut() -> String {
    "Ctrl + 4".to_string()
}

fn default_translate_shortcut() -> String {
    "Ctrl + 5".to_string()
}

fn default_summarize_shortcut() -> String {
    "Ctrl + 6".to_string()
}

fn default_confident_shortcut() -> String {
    "Ctrl + 7".to_string()
}

fn default_simplify_shortcut() -> String {
    "Ctrl + 8".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteRequest {
    pub input: String,
    pub mode: RewriteMode,
    pub target_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteResponse {
    pub input: String,
    pub output: String,
    pub mode: RewriteMode,
    pub provider: ProviderId,
    pub used_offline_fallback: bool,
    pub character_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupPayload {
    pub input: String,
    pub output: String,
    pub mode: RewriteMode,
    pub provider: ProviderId,
    pub used_offline_fallback: bool,
    pub character_count: usize,
    pub source: String,
}

impl From<(RewriteResponse, &str)> for PopupPayload {
    fn from((response, source): (RewriteResponse, &str)) -> Self {
        Self {
            input: response.input,
            output: response.output,
            mode: response.mode,
            provider: response.provider,
            used_offline_fallback: response.used_offline_fallback,
            character_count: response.character_count,
            source: source.to_string(),
        }
    }
}
