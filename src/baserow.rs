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

#[derive(Debug, Serialize)]
pub struct MediaEntry {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Author")]
    pub author: String,
    #[serde(rename = "ISBN")]
    pub isbn: Option<String>,
    #[serde(rename = "Synopsis")]
    pub synopsis: String,
    #[serde(rename = "Category")]
    pub category: Vec<u64>, // Array of category IDs
    #[serde(rename = "Read")]
    pub read: bool,
    #[serde(rename = "Rating")]
    pub rating: u32,
    #[serde(rename = "Media Type")]
    pub media_type: Option<u64>,
    #[serde(rename = "Location", skip_serializing_if = "Vec::is_empty")]
    pub location: Vec<u64>, // Array of location IDs - left empty for manual entry
    #[serde(rename = "Cover", skip_serializing_if = "Vec::is_empty")]
    pub cover: Vec<CoverImage>, // Array of cover images
    #[serde(rename = "Status")]
    pub status: u64, // Status field (3028=In Place, 3029=Active, 3030=On Loan)
}

#[derive(Debug, Serialize)]
pub struct CoverImage {
    pub name: String,
}


#[derive(Debug, Deserialize)]
pub struct FileUploadResponse {
    #[allow(dead_code)]
    pub url: String,
    pub name: String,
    #[allow(dead_code)]
    pub size: u64,
    #[allow(dead_code)]
    pub mime_type: String,
    #[allow(dead_code)]
    pub is_image: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub image_width: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    pub image_height: Option<u32>,
    #[allow(dead_code)]
    pub uploaded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatedEntry {
    pub id: u64,
    #[serde(flatten)]
    #[allow(dead_code)]
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


    pub async fn create_media_entry(&self, entry_data: MediaEntry) -> Result<CreatedEntry, BaserowError> {
        println!("Creating new media entry in Baserow...");
        
        let url = format!("{}/api/database/rows/table/{}/?user_field_names=true", 
            self.config.base_url.trim_end_matches('/'), 
            self.config.media_table_id
        );

        println!("Making request to: {}", url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Token {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(&entry_data)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(BaserowError::InvalidResponse(format!(
                "Failed to create entry: HTTP {} - {}", 
                status,
                error_text
            )));
        }

        let created_entry: CreatedEntry = response.json().await
            .map_err(|e| BaserowError::InvalidResponse(e.to_string()))?;

        println!("Successfully created entry with ID: {}", created_entry.id);
        Ok(created_entry)
    }

    pub fn find_category_ids_by_names(&self, category_names: &[String], available_categories: &[Category]) -> Vec<u64> {
        let mut category_ids = Vec::new();
        
        for name in category_names {
            if let Some(category) = available_categories.iter().find(|cat| {
                cat.get_name()
                    .map(|cat_name| cat_name.to_lowercase() == name.to_lowercase())
                    .unwrap_or(false)
            }) {
                category_ids.push(category.id);
            } else {
                println!("Warning: Category '{}' not found in available categories", name);
            }
        }
        
        category_ids
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


    pub async fn upload_file_direct(&self, image_data: Vec<u8>, filename: &str) -> Result<FileUploadResponse, BaserowError> {
        println!("Uploading cover image file directly to Baserow...");
        
        let url = format!("{}/api/user-files/upload-file/", 
            self.config.base_url.trim_end_matches('/')
        );

        // Determine MIME type from filename
        let mime_type = if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
            "image/jpeg"
        } else if filename.ends_with(".png") {
            "image/png"
        } else {
            "application/octet-stream"
        };

        // Create multipart form
        let part = reqwest::multipart::Part::bytes(image_data)
            .file_name(filename.to_string())
            .mime_str(mime_type).map_err(|e| BaserowError::InvalidResponse(format!("Invalid MIME type: {}", e)))?;

        let form = reqwest::multipart::Form::new()
            .part("file", part);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Token {}", self.config.api_token))
            .multipart(form)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let upload_response: FileUploadResponse = response.json().await
                    .map_err(|e| BaserowError::InvalidResponse(format!("Failed to parse upload response: {}", e)))?;
                
                println!("Successfully uploaded cover image file: {}", upload_response.name);
                Ok(upload_response)
            }
            reqwest::StatusCode::UNAUTHORIZED => Err(BaserowError::AuthenticationFailed),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(BaserowError::InvalidResponse(format!(
                    "Failed to upload file: HTTP {} - {}", 
                    status, 
                    error_text
                )))
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