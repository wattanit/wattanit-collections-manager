use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use dialoguer::{Select, theme::ColorfulTheme};
use crate::config::Config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GoogleBooksResponse {
    pub kind: String,
    #[serde(rename = "totalItems")]
    pub total_items: u32,
    pub items: Option<Vec<BookItem>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BookItem {
    pub kind: String,
    pub id: String,
    #[serde(rename = "etag")]
    pub etag: String,
    #[serde(rename = "selfLink")]
    pub self_link: String,
    #[serde(rename = "volumeInfo")]
    pub volume_info: VolumeInfo,
    #[serde(rename = "saleInfo")]
    pub sale_info: Option<SaleInfo>,
    #[serde(rename = "accessInfo")]
    pub access_info: Option<AccessInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeInfo {
    pub title: String,
    pub subtitle: Option<String>,
    pub authors: Option<Vec<String>>,
    pub publisher: Option<String>,
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "industryIdentifiers")]
    pub industry_identifiers: Option<Vec<IndustryIdentifier>>,
    #[serde(rename = "readingModes")]
    pub reading_modes: Option<HashMap<String, bool>>,
    #[serde(rename = "pageCount")]
    pub page_count: Option<u32>,
    #[serde(rename = "printType")]
    pub print_type: Option<String>,
    pub categories: Option<Vec<String>>,
    #[serde(rename = "maturityRating")]
    pub maturity_rating: Option<String>,
    #[serde(rename = "allowAnonLogging")]
    pub allow_anon_logging: Option<bool>,
    #[serde(rename = "contentVersion")]
    pub content_version: Option<String>,
    #[serde(rename = "panelizationSummary")]
    pub panelization_summary: Option<HashMap<String, bool>>,
    #[serde(rename = "imageLinks")]
    pub image_links: Option<ImageLinks>,
    pub language: Option<String>,
    #[serde(rename = "previewLink")]
    pub preview_link: Option<String>,
    #[serde(rename = "infoLink")]
    pub info_link: Option<String>,
    #[serde(rename = "canonicalVolumeLink")]
    pub canonical_volume_link: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndustryIdentifier {
    #[serde(rename = "type")]
    pub identifier_type: String,
    pub identifier: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageLinks {
    #[serde(rename = "smallThumbnail")]
    pub small_thumbnail: Option<String>,
    pub thumbnail: Option<String>,
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaleInfo {
    pub country: Option<String>,
    pub saleability: Option<String>,
    #[serde(rename = "isEbook")]
    pub is_ebook: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccessInfo {
    pub country: Option<String>,
    pub viewability: Option<String>,
    pub embeddable: Option<bool>,
    #[serde(rename = "publicDomain")]
    pub public_domain: Option<bool>,
}

pub struct GoogleBooksClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl GoogleBooksClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }

    pub async fn search_by_isbn(&self, isbn: &str) -> Result<GoogleBooksResponse, Box<dyn std::error::Error>> {
        let url = if self.api_key.contains("your_") || self.api_key.is_empty() {
            // Try without API key for basic usage
            format!("{}/volumes?q=isbn:{}", self.base_url, isbn)
        } else {
            format!("{}/volumes?q=isbn:{}&key={}", self.base_url, isbn, self.api_key)
        };

        println!("Making request to: {}", url.replace(&self.api_key, "***"));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(format!("Google Books API error: {} - {}", status, error_text).into());
        }

        let books_response: GoogleBooksResponse = response.json().await?;
        Ok(books_response)
    }

    pub async fn search_by_title_author(
        &self,
        title: &str,
        author: &str,
    ) -> Result<GoogleBooksResponse, Box<dyn std::error::Error>> {
        let query = format!("intitle:\"{}\" inauthor:\"{}\"", title, author);
        let url = if self.api_key.contains("your_") || self.api_key.is_empty() {
            format!(
                "{}/volumes?q={}",
                self.base_url,
                urlencoding::encode(&query)
            )
        } else {
            format!(
                "{}/volumes?q={}&key={}",
                self.base_url,
                urlencoding::encode(&query),
                self.api_key
            )
        };

        println!("Making request to: {}", url.replace(&self.api_key, "***"));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(format!("Google Books API error: {} - {}", status, error_text).into());
        }

        let books_response: GoogleBooksResponse = response.json().await?;
        Ok(books_response)
    }

    pub async fn search_by_title(&self, title: &str) -> Result<GoogleBooksResponse, Box<dyn std::error::Error>> {
        let query = format!("intitle:{}", title);
        let url = format!(
            "{}/volumes?q={}&key={}",
            self.base_url,
            urlencoding::encode(&query),
            self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Google Books API error: {}", response.status()).into());
        }

        let books_response: GoogleBooksResponse = response.json().await?;
        Ok(books_response)
    }
}

// Helper functions for extracting data from Google Books response
impl BookItem {
    pub fn get_isbn_13(&self) -> Option<String> {
        self.volume_info.industry_identifiers.as_ref()?.iter()
            .find(|id| id.identifier_type == "ISBN_13")
            .map(|id| id.identifier.clone())
    }

    pub fn get_isbn_10(&self) -> Option<String> {
        self.volume_info.industry_identifiers.as_ref()?.iter()
            .find(|id| id.identifier_type == "ISBN_10")
            .map(|id| id.identifier.clone())
    }

    pub fn get_best_cover_image(&self) -> Option<String> {
        let image_links = self.volume_info.image_links.as_ref()?;
        
        // Prefer larger images first
        image_links.extra_large.clone()
            .or_else(|| image_links.large.clone())
            .or_else(|| image_links.medium.clone())
            .or_else(|| image_links.small.clone())
            .or_else(|| image_links.thumbnail.clone())
            .or_else(|| image_links.small_thumbnail.clone())
    }

    pub fn get_primary_author(&self) -> Option<String> {
        self.volume_info.authors.as_ref()?.first().cloned()
    }

    pub fn get_all_authors(&self) -> String {
        self.volume_info.authors.as_ref()
            .map(|authors| authors.join(", "))
            .unwrap_or_else(|| "Unknown Author".to_string())
    }

    pub fn get_full_title(&self) -> String {
        match &self.volume_info.subtitle {
            Some(subtitle) => format!("{}: {}", self.volume_info.title, subtitle),
            None => self.volume_info.title.clone(),
        }
    }
}

pub fn display_google_book_info(book: &BookItem, _config: &Config) {
    println!("\n=== Book Information (Google Books) ===");
    println!("Title: {}", book.get_full_title());
    println!("Author(s): {}", book.get_all_authors());
    
    if let Some(publisher) = &book.volume_info.publisher {
        println!("Publisher: {}", publisher);
    }
    
    if let Some(date) = &book.volume_info.published_date {
        println!("Published: {}", date);
    }
    
    if let Some(page_count) = book.volume_info.page_count {
        println!("Pages: {}", page_count);
    }
    
    if let Some(isbn13) = book.get_isbn_13() {
        println!("ISBN-13: {}", isbn13);
    }
    
    if let Some(isbn10) = book.get_isbn_10() {
        println!("ISBN-10: {}", isbn10);
    }
    
    if let Some(description) = &book.volume_info.description {
        let desc = if description.len() > 200 {
            format!("{}...", &description[..200])
        } else {
            description.clone()
        };
        println!("Description: {}", desc);
    }
    
    if let Some(cover_url) = book.get_best_cover_image() {
        println!("Cover Image: {}", cover_url);
    }
    
    if let Some(categories) = &book.volume_info.categories {
        println!("Categories: {}", categories.join(", "));
    }
    
    println!("========================================\n");
}

pub fn interactive_select_google_book(books: &[BookItem]) -> Result<Option<&BookItem>, Box<dyn std::error::Error>> {
    let items: Vec<String> = books.iter().map(|book| {
        format!("{} by {} ({})", 
            book.get_full_title(), 
            book.get_all_authors(),
            book.volume_info.published_date.as_deref().unwrap_or("Unknown year")
        )
    }).collect();
    
    let mut items_with_cancel = items;
    items_with_cancel.push("Cancel - don't add any book".to_string());
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a book to add")
        .items(&items_with_cancel)
        .default(0)
        .interact()?;
    
    if selection == items_with_cancel.len() - 1 {
        // User selected cancel
        Ok(None)
    } else {
        Ok(books.get(selection))
    }
}