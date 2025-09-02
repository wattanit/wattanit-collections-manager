use async_trait::async_trait;
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum BookResult {
    Google(crate::google_books::BookItem),
    OpenLibrary(crate::open_library::OpenLibraryBook),
}

#[derive(Debug)]
pub struct SearchResults {
    pub books: Vec<BookResult>,
    pub source: String,
}

impl BookResult {
    pub fn get_full_title(&self) -> String {
        match self {
            BookResult::Google(book) => book.get_full_title(),
            BookResult::OpenLibrary(book) => book.get_full_title(),
        }
    }

    pub fn get_all_authors(&self) -> String {
        match self {
            BookResult::Google(book) => book.get_all_authors(),
            BookResult::OpenLibrary(book) => book.get_all_authors(),
        }
    }

    pub fn get_published_date(&self) -> Option<String> {
        match self {
            BookResult::Google(book) => book.volume_info.published_date.clone(),
            BookResult::OpenLibrary(book) => book.get_latest_publish_year()
                .map(|y| y.to_string())
                .or_else(|| book.get_latest_publish_date()),
        }
    }

    pub fn display_info(&self, config: &Config) -> tokio::task::JoinHandle<()> {
        match self {
            BookResult::Google(book) => {
                let book = book.clone();
                let config = config.clone();
                tokio::spawn(async move {
                    crate::google_books::display_google_book_info(&book, &config);
                })
            }
            BookResult::OpenLibrary(book) => {
                let book = book.clone();
                let config = config.clone();
                tokio::spawn(async move {
                    crate::open_library::display_open_library_book_info(&book, &config).await;
                })
            }
        }
    }
}

pub fn interactive_select_book(results: &SearchResults) -> Result<Option<&BookResult>, Box<dyn std::error::Error>> {
    use dialoguer::{Select, theme::ColorfulTheme};

    let items: Vec<String> = results.books.iter().map(|book| {
        format!("{} by {} ({})", 
            book.get_full_title(), 
            book.get_all_authors(),
            book.get_published_date().unwrap_or_else(|| "Unknown year".to_string())
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
        Ok(results.books.get(selection))
    }
}

#[async_trait]
pub trait BookSearcher {
    async fn search_by_isbn(&self, isbn: &str) -> Result<SearchResults, Box<dyn std::error::Error>>;
    async fn search_by_title_author(&self, title: &str, author: &str) -> Result<SearchResults, Box<dyn std::error::Error>>;
}

#[async_trait]
impl BookSearcher for crate::google_books::GoogleBooksClient {
    async fn search_by_isbn(&self, isbn: &str) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let response = self.search_by_isbn(isbn).await?;
        let books = response.items.unwrap_or_default()
            .into_iter()
            .map(BookResult::Google)
            .collect();
        
        Ok(SearchResults {
            books,
            source: "Google Books".to_string(),
        })
    }

    async fn search_by_title_author(&self, title: &str, author: &str) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let response = self.search_by_title_author(title, author).await?;
        let books = response.items.unwrap_or_default()
            .into_iter()
            .map(BookResult::Google)
            .collect();
        
        Ok(SearchResults {
            books,
            source: "Google Books".to_string(),
        })
    }
}

#[async_trait]
impl BookSearcher for crate::open_library::OpenLibraryClient {
    async fn search_by_isbn(&self, isbn: &str) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let response = self.search_by_isbn(isbn).await?;
        let books = response.docs
            .into_iter()
            .map(BookResult::OpenLibrary)
            .collect();
        
        Ok(SearchResults {
            books,
            source: "Open Library".to_string(),
        })
    }

    async fn search_by_title_author(&self, title: &str, author: &str) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let response = self.search_by_title_author(title, author).await?;
        let books = response.docs
            .into_iter()
            .map(BookResult::OpenLibrary)
            .collect();
        
        Ok(SearchResults {
            books,
            source: "Open Library".to_string(),
        })
    }
}

pub struct CombinedBookSearcher {
    google_client: crate::google_books::GoogleBooksClient,
    open_library_client: crate::open_library::OpenLibraryClient,
    baserow_client: crate::baserow::BaserowClient,
    config: Config,
}

impl CombinedBookSearcher {
    pub fn new(
        google_client: crate::google_books::GoogleBooksClient,
        open_library_client: crate::open_library::OpenLibraryClient,
        baserow_client: crate::baserow::BaserowClient,
        config: Config,
    ) -> Self {
        Self {
            google_client,
            open_library_client,
            baserow_client,
            config,
        }
    }

    pub async fn search_by_isbn(&self, isbn: &str) -> Result<Option<BookResult>, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Fetching book data from Google Books API...");
        }
        
        // Try Google Books first
        match BookSearcher::search_by_isbn(&self.google_client, isbn).await {
            Ok(results) if !results.books.is_empty() => {
                return self.handle_search_results(results, isbn).await;
            }
            Ok(_) => {
                if self.config.app.verbose {
                    println!("No results from Google Books API, trying Open Library...");
                }
            }
            Err(e) => {
                if self.config.app.verbose {
                    println!("Google Books API error: {}, trying Open Library...", e);
                }
            }
        }
        
        // Fallback to Open Library
        if self.config.app.verbose {
            println!("Fetching book data from Open Library API...");
        }
        
        let results = BookSearcher::search_by_isbn(&self.open_library_client, isbn).await?;
        
        if results.books.is_empty() {
            println!("No books found for ISBN: {} in either Google Books or Open Library", isbn);
            return Ok(None);
        }
        
        self.handle_search_results(results, isbn).await
    }

    pub async fn search_by_title_author(&self, title: &str, author: &str) -> Result<Option<BookResult>, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Searching for books on Google Books API...");
        }
        
        // Try Google Books first
        match BookSearcher::search_by_title_author(&self.google_client, title, author).await {
            Ok(results) if !results.books.is_empty() => {
                return self.handle_search_results(results, &format!("title: '{}', author: '{}'", title, author)).await;
            }
            Ok(_) => {
                if self.config.app.verbose {
                    println!("No results from Google Books API, trying Open Library...");
                }
            }
            Err(e) => {
                if self.config.app.verbose {
                    println!("Google Books API error: {}, trying Open Library...", e);
                }
            }
        }
        
        // Fallback to Open Library
        if self.config.app.verbose {
            println!("Searching for books on Open Library API...");
        }
        
        let results = BookSearcher::search_by_title_author(&self.open_library_client, title, author).await?;
        
        if results.books.is_empty() {
            println!("No books found for title: '{}' and author: '{}' in either Google Books or Open Library", title, author);
            return Ok(None);
        }
        
        self.handle_search_results(results, &format!("title: '{}', author: '{}'", title, author)).await
    }

    async fn handle_search_results(&self, results: SearchResults, search_query: &str) -> Result<Option<BookResult>, Box<dyn std::error::Error>> {
        let selected_book = if results.books.len() > 1 {
            // Limit to max_search_results for display
            let display_books = if results.books.len() > self.config.app.max_search_results {
                &results.books[..self.config.app.max_search_results]
            } else {
                &results.books
            };
            
            let truncated_results = SearchResults {
                books: display_books.to_vec(),
                source: results.source.clone(),
            };
            
            println!("Found {} books from {} for {} (showing top {}):", 
                results.books.len(), results.source, search_query, display_books.len());
            
            match interactive_select_book(&truncated_results) {
                Ok(Some(selected_book)) => Some(selected_book.clone()),
                Ok(None) => {
                    println!("No book selected.");
                    return Ok(None);
                }
                Err(e) => {
                    if self.config.app.verbose {
                        println!("Error in interactive selection: {}", e);
                    }
                    // Fall through to show first result
                    results.books.first().cloned()
                }
            }
        } else {
            results.books.first().cloned()
        };
        
        if let Some(book) = selected_book {
            // Display book information
            let handle = book.display_info(&self.config);
            handle.await?;
            
            // Fetch categories from Baserow
            match self.baserow_client.fetch_categories().await {
                Ok(categories) => {
                    if !categories.is_empty() {
                        crate::baserow::display_categories(&categories);
                        // TODO: This is where we'll add LLM category selection in the next step
                        println!("Categories fetched successfully! Next: implement LLM category selection.");
                    } else {
                        println!("No categories found in Baserow table.");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch categories from Baserow: {}", e);
                    if self.config.app.verbose {
                        eprintln!("Make sure your Baserow API token and categories table ID are correct.");
                    }
                }
            }
            
            return Ok(Some(book));
        }
        
        Ok(None)
    }
}