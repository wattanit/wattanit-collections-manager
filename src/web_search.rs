use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WebSearchClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DuckDuckGoResponse {
    #[serde(rename = "RelatedTopics")]
    pub related_topics: Vec<DuckDuckGoTopic>,
    #[serde(rename = "Abstract")]
    pub abstract_text: String,
    #[serde(rename = "AbstractText")]
    pub abstract_text_plain: String,
    #[serde(rename = "AbstractSource")]
    pub abstract_source: String,
    #[serde(rename = "AbstractURL")]
    pub abstract_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DuckDuckGoTopic {
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "FirstURL")]
    pub first_url: Option<String>,
}

#[derive(Debug)]
pub enum SearchError {
    RequestFailed(reqwest::Error),
    ParseError(String),
    NoResults,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SearchError::RequestFailed(e) => write!(f, "Search request failed: {}", e),
            SearchError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SearchError::NoResults => write!(f, "No search results found"),
        }
    }
}

impl std::error::Error for SearchError {}

impl From<reqwest::Error> for SearchError {
    fn from(error: reqwest::Error) -> Self {
        SearchError::RequestFailed(error)
    }
}

impl WebSearchClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();
        
        Self { client }
    }

    pub async fn search_book_info(&self, title: &str, author: &str) -> Result<Vec<SearchResult>, SearchError> {
        println!("Searching web for additional book information...");
        
        // Try DuckDuckGo instant answer API first
        if let Ok(results) = self.search_duckduckgo(title, author).await {
            if !results.is_empty() {
                return Ok(results);
            }
        }

        // Fallback to basic web search
        self.search_basic(title, author).await
    }

    async fn search_duckduckgo(&self, title: &str, author: &str) -> Result<Vec<SearchResult>, SearchError> {
        let query = format!("{} by {} book synopsis review", title, author);
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_redirect=1&no_html=1&skip_disambig=1",
            urlencoding::encode(&query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SearchError::NoResults);
        }

        let ddg_response: DuckDuckGoResponse = response.json().await
            .map_err(|e| SearchError::ParseError(e.to_string()))?;

        let mut results = Vec::new();

        // Add abstract if available
        if !ddg_response.abstract_text_plain.is_empty() {
            results.push(SearchResult {
                title: format!("{} - {}", title, ddg_response.abstract_source),
                url: ddg_response.abstract_url,
                snippet: ddg_response.abstract_text_plain,
            });
        }

        // Add related topics
        for topic in ddg_response.related_topics.iter().take(3) {
            if !topic.text.is_empty() {
                results.push(SearchResult {
                    title: format!("Related: {}", title),
                    url: topic.first_url.clone().unwrap_or_default(),
                    snippet: topic.text.clone(),
                });
            }
        }

        Ok(results)
    }

    async fn search_basic(&self, title: &str, author: &str) -> Result<Vec<SearchResult>, SearchError> {
        // This is a placeholder for basic search functionality
        // In a real implementation, you might use:
        // - SerpAPI (requires API key)
        // - Bing Search API (requires API key) 
        // - Custom scraping (be careful about rate limits)
        
        println!("DuckDuckGo search didn't return results, trying basic search...");
        
        // For now, return a minimal result to indicate we tried
        let basic_result = SearchResult {
            title: format!("{} by {}", title, author),
            url: String::new(),
            snippet: format!("Additional information needed for {} by {}. Consider checking Goodreads, Wikipedia, or publisher websites for detailed synopsis and genre information.", title, author),
        };

        Ok(vec![basic_result])
    }

    pub fn format_search_results(&self, results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No additional information found from web search.".to_string();
        }

        let mut formatted = String::from("=== Additional Information from Web Search ===\n");
        
        for (i, result) in results.iter().enumerate() {
            formatted.push_str(&format!(
                "\n{}. {}\n   {}\n   Source: {}\n",
                i + 1,
                result.title,
                result.snippet,
                if result.url.is_empty() { "N/A" } else { &result.url }
            ));
        }

        formatted.push_str("\n=== End of Web Search Results ===\n");
        formatted
    }
}

pub async fn enhance_book_info_with_search(
    title: &str,
    author: &str,
    existing_description: &str,
) -> String {
    let search_client = WebSearchClient::new();
    
    match search_client.search_book_info(title, author).await {
        Ok(results) => {
            let mut enhanced_info = String::new();
            enhanced_info.push_str("=== Original Book Information ===\n");
            enhanced_info.push_str(&format!("Title: {}\n", title));
            enhanced_info.push_str(&format!("Author: {}\n", author));
            enhanced_info.push_str(&format!("Description: {}\n", existing_description));
            enhanced_info.push('\n');
            enhanced_info.push_str(&search_client.format_search_results(&results));
            enhanced_info
        }
        Err(e) => {
            println!("Web search failed: {}", e);
            format!(
                "=== Book Information (Web Search Failed) ===\nTitle: {}\nAuthor: {}\nDescription: {}\n\nNote: Unable to fetch additional information from web search.",
                title, author, existing_description
            )
        }
    }
}