use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use dialoguer::{Select, theme::ColorfulTheme};
use crate::config::Config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibrarySearchResponse {
    #[serde(rename = "numFound")]
    pub num_found: u32,
    pub start: u32,
    #[serde(rename = "numFoundExact")]
    pub num_found_exact: Option<bool>,
    pub docs: Vec<OpenLibraryBook>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryBook {
    pub key: String,
    pub title: String,
    pub subtitle: Option<String>,
    #[serde(rename = "author_name")]
    pub author_name: Option<Vec<String>>,
    #[serde(rename = "author_key")]
    pub author_key: Option<Vec<String>>,
    #[serde(rename = "first_publish_year")]
    pub first_publish_year: Option<u32>,
    #[serde(rename = "publish_year")]
    pub publish_year: Option<Vec<u32>>,
    #[serde(rename = "publish_date")]
    pub publish_date: Option<Vec<String>>,
    pub publisher: Option<Vec<String>>,
    #[serde(rename = "number_of_pages_median")]
    pub number_of_pages_median: Option<u32>,
    #[serde(rename = "isbn")]
    pub isbn: Option<Vec<String>>,
    #[serde(rename = "cover_i")]
    pub cover_i: Option<u32>,
    #[serde(rename = "cover_edition_key")]
    pub cover_edition_key: Option<String>,
    #[serde(rename = "has_fulltext")]
    pub has_fulltext: Option<bool>,
    pub subject: Option<Vec<String>>,
    #[serde(rename = "subject_key")]
    pub subject_key: Option<Vec<String>>,
    pub language: Option<Vec<String>>,
    #[serde(rename = "edition_count")]
    pub edition_count: Option<u32>,
    #[serde(rename = "edition_key")]
    pub edition_key: Option<Vec<String>>,
    #[serde(rename = "first_sentence")]
    pub first_sentence: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryBookDetails {
    pub key: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<OpenLibraryDescription>,
    pub authors: Option<Vec<OpenLibraryAuthorRef>>,
    #[serde(rename = "publish_date")]
    pub publish_date: Option<String>,
    pub publishers: Option<Vec<String>>,
    #[serde(rename = "number_of_pages")]
    pub number_of_pages: Option<u32>,
    pub isbn_10: Option<Vec<String>>,
    pub isbn_13: Option<Vec<String>>,
    pub covers: Option<Vec<u32>>,
    pub subjects: Option<Vec<String>>,
    pub languages: Option<Vec<OpenLibraryLanguageRef>>,
    #[serde(rename = "works")]
    pub works: Option<Vec<OpenLibraryWorkRef>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum OpenLibraryDescription {
    String(String),
    Object {
        #[serde(rename = "type")]
        desc_type: String,
        value: String,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryAuthorRef {
    pub key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryWorkRef {
    pub key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryLanguageRef {
    pub key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenLibraryAuthor {
    pub key: String,
    pub name: String,
    #[serde(rename = "personal_name")]
    pub personal_name: Option<String>,
    #[serde(rename = "birth_date")]
    pub birth_date: Option<String>,
    #[serde(rename = "death_date")]
    pub death_date: Option<String>,
}

pub struct OpenLibraryClient {
    client: reqwest::Client,
    base_url: String,
}

impl OpenLibraryClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn search_by_isbn(&self, isbn: &str) -> Result<OpenLibrarySearchResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/search.json?isbn={}", self.base_url, isbn);

        println!("Making Open Library request to: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(format!("Open Library API error: {} - {}", status, error_text).into());
        }

        let search_response: OpenLibrarySearchResponse = response.json().await?;
        Ok(search_response)
    }

    pub async fn search_by_title_author(
        &self,
        title: &str,
        author: &str,
    ) -> Result<OpenLibrarySearchResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/search.json?title={}&author={}",
            self.base_url,
            urlencoding::encode(title),
            urlencoding::encode(author)
        );

        println!("Making Open Library request to: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(format!("Open Library API error: {} - {}", status, error_text).into());
        }

        let search_response: OpenLibrarySearchResponse = response.json().await?;
        Ok(search_response)
    }

    pub async fn get_book_details(&self, key: &str) -> Result<OpenLibraryBookDetails, Box<dyn std::error::Error>> {
        let url = format!("{}{}.json", self.base_url, key);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(format!("Open Library API error: {} - {}", status, error_text).into());
        }

        let book_details: OpenLibraryBookDetails = response.json().await?;
        Ok(book_details)
    }

    pub async fn get_author(&self, key: &str) -> Result<OpenLibraryAuthor, Box<dyn std::error::Error>> {
        let url = format!("{}{}.json", self.base_url, key);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Open Library API error: {}", response.status()).into());
        }

        let author: OpenLibraryAuthor = response.json().await?;
        Ok(author)
    }
}

// Helper functions for extracting data from Open Library response
impl OpenLibraryBook {
    pub fn get_best_isbn(&self) -> Option<String> {
        self.isbn.as_ref()?.first().cloned()
    }

    pub fn get_cover_url(&self) -> Option<String> {
        self.cover_i.map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id))
    }

    pub fn get_primary_author(&self) -> Option<String> {
        self.author_name.as_ref()?.first().cloned()
    }

    pub fn get_all_authors(&self) -> String {
        self.author_name.as_ref()
            .map(|authors| authors.join(", "))
            .unwrap_or_else(|| "Unknown Author".to_string())
    }

    pub fn get_full_title(&self) -> String {
        match &self.subtitle {
            Some(subtitle) => format!("{}: {}", self.title, subtitle),
            None => self.title.clone(),
        }
    }

    pub fn get_primary_publisher(&self) -> Option<String> {
        self.publisher.as_ref()?.first().cloned()
    }

    pub fn get_latest_publish_date(&self) -> Option<String> {
        self.publish_date.as_ref()?.first().cloned()
    }

    pub fn get_latest_publish_year(&self) -> Option<u32> {
        self.publish_year.as_ref()?.iter().max().copied()
            .or(self.first_publish_year)
    }
}

impl OpenLibraryBookDetails {
    pub fn get_description(&self) -> Option<String> {
        match &self.description {
            Some(OpenLibraryDescription::String(desc)) => Some(desc.clone()),
            Some(OpenLibraryDescription::Object { value, .. }) => Some(value.clone()),
            None => None,
        }
    }

    pub fn get_cover_url(&self) -> Option<String> {
        self.covers.as_ref()?.first()
            .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id))
    }

    pub fn get_isbn_13(&self) -> Option<String> {
        self.isbn_13.as_ref()?.first().cloned()
    }

    pub fn get_isbn_10(&self) -> Option<String> {
        self.isbn_10.as_ref()?.first().cloned()
    }

    pub fn get_full_title(&self) -> String {
        match &self.subtitle {
            Some(subtitle) => format!("{}: {}", self.title, subtitle),
            None => self.title.clone(),
        }
    }
}

pub async fn display_open_library_book_info(book: &OpenLibraryBook, _config: &Config) {
    println!("\n=== Book Information (Open Library) ===");
    println!("Title: {}", book.get_full_title());
    println!("Author(s): {}", book.get_all_authors());
    
    if let Some(publisher) = book.get_primary_publisher() {
        println!("Publisher: {}", publisher);
    }
    
    if let Some(year) = book.get_latest_publish_year() {
        println!("Published: {}", year);
    } else if let Some(date) = book.get_latest_publish_date() {
        println!("Published: {}", date);
    }
    
    if let Some(pages) = book.number_of_pages_median {
        println!("Pages: {}", pages);
    }
    
    if let Some(isbn) = book.get_best_isbn() {
        println!("ISBN: {}", isbn);
    }
    
    if let Some(cover_url) = book.get_cover_url() {
        println!("Cover Image: {}", cover_url);
    }
    
    if let Some(subjects) = &book.subject {
        let subjects_str = subjects.iter().take(5).cloned().collect::<Vec<String>>().join(", ");
        println!("Subjects: {}", subjects_str);
    }
    
    if let Some(first_sentence) = &book.first_sentence {
        if let Some(sentence) = first_sentence.first() {
            let desc = if sentence.len() > 1000 {
                format!("{}...", &sentence[..1000])
            } else {
                sentence.clone()
            };
            println!("First Sentence: {}", desc);
        }
    }
    
    println!("========================================\n");
}

pub fn interactive_select_open_library_book(books: &[OpenLibraryBook]) -> Result<Option<&OpenLibraryBook>, Box<dyn std::error::Error>> {
    let items: Vec<String> = books.iter().map(|book| {
        let year = book.get_latest_publish_year()
            .map(|y| y.to_string())
            .or_else(|| book.get_latest_publish_date())
            .unwrap_or_else(|| "Unknown year".to_string());
        
        format!("{} by {} ({})", 
            book.get_full_title(), 
            book.get_all_authors(),
            year
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