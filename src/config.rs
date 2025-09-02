use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub google_books: GoogleBooksConfig,
    pub open_library: OpenLibraryConfig,
    pub baserow: BaserowConfig,
    pub llm: LlmConfig,
    pub app: AppConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GoogleBooksConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryConfig {
    pub base_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaserowConfig {
    pub api_token: String,
    pub base_url: String,
    pub database_id: u64,
    pub media_table_id: u64,
    pub categories_table_id: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub openai: OpenAiConfig,
    pub anthropic: AnthropicConfig,
    pub ollama: OllamaConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub verbose: bool,
    pub max_search_results: usize,
    pub min_synopsis_words: usize,
    pub target_synopsis_words: usize,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();
        
        let mut config = config::Config::builder();
        
        // Start with config.yaml if it exists
        if Path::new("config.yaml").exists() {
            config = config.add_source(config::File::with_name("config"));
        }
        
        // Override with environment variables
        config = config.add_source(
            config::Environment::with_prefix("WCM")
                .prefix_separator("_")
                .separator("__")
        );
        
        let settings = config.build()?;
        let mut cfg: Config = settings.try_deserialize()?;
        
        // Override specific fields with environment variables that don't follow the nested structure
        if let Ok(api_key) = std::env::var("GOOGLE_BOOKS_API_KEY") {
            cfg.google_books.api_key = api_key;
        }
        
        if let Ok(token) = std::env::var("BASEROW_API_TOKEN") {
            cfg.baserow.api_token = token;
        }
        
        if let Ok(db_id) = std::env::var("BASEROW_DATABASE_ID") {
            cfg.baserow.database_id = db_id.parse().unwrap_or(cfg.baserow.database_id);
        }
        
        if let Ok(table_id) = std::env::var("BASEROW_MEDIA_TABLE_ID") {
            cfg.baserow.media_table_id = table_id.parse().unwrap_or(cfg.baserow.media_table_id);
        }
        
        if let Ok(table_id) = std::env::var("BASEROW_CATEGORIES_TABLE_ID") {
            cfg.baserow.categories_table_id = table_id.parse().unwrap_or(cfg.baserow.categories_table_id);
        }
        
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            cfg.llm.openai.api_key = api_key;
        }
        
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            cfg.llm.anthropic.api_key = api_key;
        }
        
        if let Ok(provider) = std::env::var("WCM_LLM_PROVIDER") {
            cfg.llm.provider = provider;
        }
        
        Ok(cfg)
    }
    
    pub fn validate(&self) -> Result<(), String> {
        // Check required API keys based on selected LLM provider
        match self.llm.provider.as_str() {
            "openai" => {
                if self.llm.openai.api_key.contains("your_") {
                    return Err("OpenAI API key not configured".to_string());
                }
            }
            "anthropic" => {
                if self.llm.anthropic.api_key.contains("your_") {
                    return Err("Anthropic API key not configured".to_string());
                }
            }
            "ollama" => {
                // No API key needed for Ollama
            }
            _ => {
                return Err(format!("Unsupported LLM provider: {}", self.llm.provider));
            }
        }
        
        // Google Books API key is optional for basic usage
        // if self.google_books.api_key.contains("your_") {
        //     return Err("Google Books API key not configured".to_string());
        // }
        
        // Check Baserow configuration
        if self.baserow.api_token.contains("your_") {
            return Err("Baserow API token not configured".to_string());
        }
        
        Ok(())
    }
}