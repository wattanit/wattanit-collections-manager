use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::BaserowConfig;

#[derive(Debug, Clone)]
pub struct BaserowClient {
    client: reqwest::Client,
    config: BaserowConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaserowResponse<T> {
    pub count: Option<u32>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Category {
    pub id: u64,
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

impl Category {
    pub fn get_name(&self) -> Option<String> {
        // Try common field names for category name
        self.fields.get("Name")
            .or_else(|| self.fields.get("name"))
            .or_else(|| self.fields.get("Category"))
            .or_else(|| self.fields.get("category"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_description(&self) -> Option<String> {
        self.fields.get("Description")
            .or_else(|| self.fields.get("description"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

#[derive(Debug)]
pub enum BaserowError {
    RequestFailed(reqwest::Error),
    InvalidResponse(String),
    AuthenticationFailed,
    NotFound,
}

impl std::fmt::Display for BaserowError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BaserowError::RequestFailed(e) => write!(f, "Request failed: {}", e),
            BaserowError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            BaserowError::AuthenticationFailed => write!(f, "Authentication failed"),
            BaserowError::NotFound => write!(f, "Resource not found"),
        }
    }
}

impl std::error::Error for BaserowError {}

impl From<reqwest::Error> for BaserowError {
    fn from(error: reqwest::Error) -> Self {
        BaserowError::RequestFailed(error)
    }
}

impl BaserowClient {
    pub fn new(config: BaserowConfig) -> Self {
        let client = reqwest::Client::new();
        Self { client, config }
    }

    async fn make_request<T>(&self, endpoint: &str) -> Result<T, BaserowError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/api/database/rows/table/{}/?user_field_names=true", 
            self.config.base_url.trim_end_matches('/'), 
            endpoint
        );

        println!("Making request to: {}", url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Token {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let text = response.text().await?;
                serde_json::from_str(&text).map_err(|e| {
                    BaserowError::InvalidResponse(format!("Failed to parse JSON: {}", e))
                })
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(BaserowError::AuthenticationFailed),
            reqwest::StatusCode::NOT_FOUND => Err(BaserowError::NotFound),
            status => Err(BaserowError::InvalidResponse(format!("HTTP {}", status))),
        }
    }

    pub async fn fetch_categories(&self) -> Result<Vec<Category>, BaserowError> {
        println!("Fetching categories from Baserow...");
        
        let response: BaserowResponse<Category> = self
            .make_request(&self.config.categories_table_id.to_string())
            .await?;

        println!("Found {} categories", response.results.len());
        Ok(response.results)
    }

    pub async fn test_connection(&self) -> Result<(), BaserowError> {
        println!("Testing Baserow connection...");
        
        let url = format!("{}/api/database/rows/table/{}/?user_field_names=true&size=1", 
            self.config.base_url.trim_end_matches('/'), 
            self.config.categories_table_id
        );

        println!("Testing URL: {}", url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Token {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                println!("Baserow connection successful!");
                Ok(())
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                println!("Authentication failed - check your API token");
                Err(BaserowError::AuthenticationFailed)
            }
            reqwest::StatusCode::NOT_FOUND => {
                println!("Categories table not found - check your table ID");
                Err(BaserowError::NotFound)
            }
            status => {
                println!("Connection failed with status: {}", status);
                let text = response.text().await.unwrap_or_default();
                if !text.is_empty() {
                    println!("Response body: {}", text);
                }
                Err(BaserowError::InvalidResponse(format!("HTTP {}", status)))
            }
        }
    }
}

pub fn display_categories(categories: &[Category]) {
    if categories.is_empty() {
        println!("No categories found");
        return;
    }

    println!("\nAvailable categories:");
    for (index, category) in categories.iter().enumerate() {
        let name = category.get_name().unwrap_or_else(|| format!("Category {}", category.id));
        let description = category.get_description()
            .map(|d| format!(" - {}", d))
            .unwrap_or_default();
        
        println!("  {}. {}{}", index + 1, name, description);
    }
    println!();
}