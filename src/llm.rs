use reqwest;
use serde::{Deserialize, Serialize};
use crate::config::{Config, LlmConfig};
use crate::baserow::Category;

#[derive(Debug, Clone)]
pub enum LlmProvider {
    Ollama(OllamaClient),
    OpenAi(OpenAiClient),
    Anthropic(AnthropicClient),
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

#[derive(Debug, Clone)]
pub struct OpenAiClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnthropicClient {
    #[allow(dead_code)]
    client: reqwest::Client,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    model: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiChoice {
    pub message: OpenAiMessage,
}

#[derive(Debug)]
pub enum LlmError {
    RequestFailed(reqwest::Error),
    InvalidResponse(String),
    #[allow(dead_code)]
    ModelNotAvailable,
    ConfigurationError(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LlmError::RequestFailed(e) => write!(f, "LLM request failed: {}", e),
            LlmError::InvalidResponse(msg) => write!(f, "Invalid LLM response: {}", msg),
            LlmError::ModelNotAvailable => write!(f, "LLM model not available"),
            LlmError::ConfigurationError(msg) => write!(f, "LLM configuration error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<reqwest::Error> for LlmError {
    fn from(error: reqwest::Error) -> Self {
        LlmError::RequestFailed(error)
    }
}

impl LlmProvider {
    pub fn from_config(config: &Config) -> Result<Self, LlmError> {
        match config.llm.provider.as_str() {
            "ollama" => Ok(LlmProvider::Ollama(OllamaClient::new(&config.llm)?)),
            "openai" => Ok(LlmProvider::OpenAi(OpenAiClient::new(&config.llm)?)),
            "anthropic" => Ok(LlmProvider::Anthropic(AnthropicClient::new(&config.llm)?)),
            provider => Err(LlmError::ConfigurationError(format!(
                "Unsupported LLM provider: {}. Supported providers: ollama, openai, anthropic", 
                provider
            ))),
        }
    }

    pub async fn select_categories(
        &self,
        book_info: &str,
        available_categories: &[Category],
    ) -> Result<Vec<String>, LlmError> {
        let prompt = create_category_selection_prompt(book_info, available_categories);
        
        match self {
            LlmProvider::Ollama(client) => client.generate_response(&prompt).await,
            LlmProvider::OpenAi(client) => client.generate_response(&prompt).await,
            LlmProvider::Anthropic(client) => client.generate_response(&prompt).await,
        }
        .and_then(|response| parse_category_response(&response, available_categories))
    }

    pub async fn generate_synopsis(
        &self,
        book_info: &str,
        target_words: usize,
    ) -> Result<String, LlmError> {
        let prompt = create_synopsis_prompt(book_info, target_words);
        
        let response = match self {
            LlmProvider::Ollama(client) => client.generate_text(&prompt).await?,
            LlmProvider::OpenAi(client) => client.generate_text(&prompt).await?,
            LlmProvider::Anthropic(client) => client.generate_text(&prompt).await?,
        };
        
        // Clean up the response by removing redundant "Synopsis" prefix
        let cleaned_response = response
            .trim()
            .strip_prefix("**SYNOPSIS**")
            .or_else(|| response.strip_prefix("SYNOPSIS:"))
            .or_else(|| response.strip_prefix("Synopsis:"))
            .or_else(|| response.strip_prefix("**Synopsis**"))
            .unwrap_or(&response)
            .trim();
        
        Ok(cleaned_response.to_string())
    }
}

impl OllamaClient {
    pub fn new(config: &LlmConfig) -> Result<Self, LlmError> {
        let client = reqwest::Client::new();
        Ok(Self {
            client,
            base_url: config.ollama.base_url.clone(),
            model: config.ollama.model.clone(),
        })
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String, LlmError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LlmError::InvalidResponse(format!(
                "Ollama API returned status: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaResponse = response.json().await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        Ok(ollama_response.response)
    }

    pub async fn generate_text(&self, prompt: &str) -> Result<String, LlmError> {
        self.generate_response(prompt).await
    }
}

impl OpenAiClient {
    pub fn new(config: &LlmConfig) -> Result<Self, LlmError> {
        if config.openai.api_key.contains("your_") {
            return Err(LlmError::ConfigurationError(
                "OpenAI API key not configured".to_string()
            ));
        }

        let client = reqwest::Client::new();
        Ok(Self {
            client,
            api_key: config.openai.api_key.clone(),
            base_url: config.openai.base_url.clone(),
            model: config.openai.model.clone(),
        })
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String, LlmError> {
        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(1000),
            temperature: Some(0.7),
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LlmError::InvalidResponse(format!(
                "OpenAI API returned status: {}",
                response.status()
            )));
        }

        let openai_response: OpenAiResponse = response.json().await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        if let Some(choice) = openai_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(LlmError::InvalidResponse("No response from OpenAI".to_string()))
        }
    }

    pub async fn generate_text(&self, prompt: &str) -> Result<String, LlmError> {
        self.generate_response(prompt).await
    }
}

impl AnthropicClient {
    pub fn new(config: &LlmConfig) -> Result<Self, LlmError> {
        if config.anthropic.api_key.contains("your_") {
            return Err(LlmError::ConfigurationError(
                "Anthropic API key not configured".to_string()
            ));
        }

        let client = reqwest::Client::new();
        Ok(Self {
            client,
            api_key: config.anthropic.api_key.clone(),
            base_url: config.anthropic.base_url.clone(),
            model: config.anthropic.model.clone(),
        })
    }

    pub async fn generate_response(&self, _prompt: &str) -> Result<String, LlmError> {
        // Placeholder for Anthropic implementation
        // Would need to implement Claude API calls here
        Err(LlmError::ConfigurationError(
            "Anthropic client not yet implemented".to_string()
        ))
    }

    pub async fn generate_text(&self, prompt: &str) -> Result<String, LlmError> {
        self.generate_response(prompt).await
    }
}

fn create_category_selection_prompt(book_info: &str, categories: &[Category]) -> String {
    let category_list = categories
        .iter()
        .filter_map(|cat| cat.get_name())
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        r#"You are a librarian helping to categorize books. Based on the book information provided, select 3-5 categories that best describe this book.

BOOK INFORMATION:
{}

AVAILABLE CATEGORIES (you MUST choose ONLY from these exact categories):
{}

INSTRUCTIONS:
1. Select 3-5 categories from the list above that best fit this book
2. Consider genre, subject matter, target audience, and content type
3. Return ONLY the category names, separated by commas
4. Use the exact category names as listed above
5. Do not create new categories or modify existing ones

RESPONSE FORMAT: Category1, Category2, Category3, Category4, Category5"#,
        book_info,
        category_list
    )
}

fn create_synopsis_prompt(book_info: &str, target_words: usize) -> String {
    format!(
        r#"Based on the book information provided, write a comprehensive synopsis of approximately {} words.

BOOK INFORMATION:
{}

INSTRUCTIONS:
1. Write a clear, engaging synopsis that captures the book's essence
2. Include main themes, plot elements (without major spoilers), and key characters
3. Target length: approximately {} words
4. Write in an informative yet engaging style suitable for a library catalog
5. Focus on what makes this book unique and interesting to potential readers

SYNOPSIS:"#,
        target_words,
        book_info,
        target_words
    )
}

fn parse_category_response(response: &str, available_categories: &[Category]) -> Result<Vec<String>, LlmError> {
    let available_names: Vec<String> = available_categories
        .iter()
        .filter_map(|cat| cat.get_name())
        .map(|name| name.to_lowercase())
        .collect();

    let selected_categories: Vec<String> = response
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|category| {
            available_names.contains(&category.to_lowercase())
        })
        .take(5) // Limit to maximum 5 categories
        .collect();

    if selected_categories.is_empty() {
        Err(LlmError::InvalidResponse(
            "No valid categories found in LLM response".to_string()
        ))
    } else {
        Ok(selected_categories)
    }
}