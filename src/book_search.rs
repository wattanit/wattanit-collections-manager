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

    pub async fn search_by_isbn(&self, isbn: &str, is_ebook: bool) -> Result<Option<BookResult>, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Fetching book data from Google Books API...");
        }
        
        // Try Google Books first
        match BookSearcher::search_by_isbn(&self.google_client, isbn).await {
            Ok(results) if !results.books.is_empty() => {
                return self.handle_search_results(results, isbn, is_ebook).await;
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
        
        self.handle_search_results(results, isbn, is_ebook).await
    }

    pub async fn search_by_title_author(&self, title: &str, author: &str, is_ebook: bool) -> Result<Option<BookResult>, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Searching for books on Google Books API...");
        }
        
        // Try Google Books first
        match BookSearcher::search_by_title_author(&self.google_client, title, author).await {
            Ok(results) if !results.books.is_empty() => {
                return self.handle_search_results(results, &format!("title: '{}', author: '{}'", title, author), is_ebook).await;
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
        
        self.handle_search_results(results, &format!("title: '{}', author: '{}'", title, author), is_ebook).await
    }

    async fn handle_search_results(&self, results: SearchResults, search_query: &str, is_ebook: bool) -> Result<Option<BookResult>, Box<dyn std::error::Error>> {
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
                        if self.config.app.verbose {
                            crate::baserow::display_categories(&categories);
                        }
                        
                        // Perform LLM-powered category selection
                        match self.select_categories_with_llm(&book, &categories).await {
                            Ok(selected_categories) => {
                                println!("Selected categories: {}", selected_categories.join(", "));
                                
                                // Check if synopsis needs to be generated
                                let final_synopsis = match self.generate_synopsis_if_needed(&book).await {
                                    Ok(Some(synopsis)) => {
                                        println!("\n=== Generated Synopsis ===");
                                        println!("{}", synopsis);
                                        println!("========================\n");
                                        synopsis
                                    }
                                    Ok(None) => {
                                        if self.config.app.verbose {
                                            println!("Existing synopsis is sufficient, no LLM generation needed.");
                                        }
                                        // Use existing description as synopsis
                                        match &book {
                                            BookResult::Google(google_book) => {
                                                google_book.volume_info.description.as_deref().unwrap_or("No description available").to_string()
                                            }
                                            BookResult::OpenLibrary(_) => "No description available".to_string(),
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to generate synopsis: {}", e);
                                        // Use existing description as fallback
                                        match &book {
                                            BookResult::Google(google_book) => {
                                                google_book.volume_info.description.as_deref().unwrap_or("No description available").to_string()
                                            }
                                            BookResult::OpenLibrary(_) => "No description available".to_string(),
                                        }
                                    }
                                };
                                
                                // Display pre-flight confirmation
                                if !self.show_preflight_confirmation(&book, &selected_categories, &final_synopsis, is_ebook)? {
                                    println!("Operation cancelled by user.");
                                    return Ok(Some(book));
                                }
                                
                                // Handle cover image upload after confirmation
                                let cover_images = self.handle_cover_image_upload(&book).await;
                                
                                // Create Baserow entry with all the collected data
                                match self.create_baserow_entry(&book, &selected_categories, &final_synopsis, &categories, is_ebook, cover_images).await {
                                    Ok(entry_id) => {
                                        println!("✅ Successfully added book to library! Entry ID: {}", entry_id);
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to create Baserow entry: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to select categories with LLM: {}", e);
                                println!("Available categories:");
                                crate::baserow::display_categories(&categories);
                            }
                        }
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

    async fn select_categories_with_llm(
        &self,
        book: &BookResult,
        categories: &[crate::baserow::Category],
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Enhancing book information with web search...");
        }

        // Get basic book information
        let title = book.get_full_title();
        let author = book.get_all_authors();
        let existing_description = match book {
            BookResult::Google(google_book) => {
                google_book.volume_info.description.as_deref().unwrap_or("No description available")
            }
            BookResult::OpenLibrary(_) => "No description available",
        };

        // Enhance with web search
        let enhanced_info = crate::web_search::enhance_book_info_with_search(
            &title,
            &author,
            existing_description,
        ).await;

        if self.config.app.verbose {
            println!("Enhanced book information prepared, consulting LLM for category selection...");
        }

        // Use LLM to select categories
        let llm_provider = crate::llm::LlmProvider::from_config(&self.config)?;
        let selected_categories = llm_provider.select_categories(&enhanced_info, categories).await?;

        Ok(selected_categories)
    }

    async fn generate_synopsis_if_needed(
        &self,
        book: &BookResult,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let existing_description = match book {
            BookResult::Google(google_book) => {
                google_book.volume_info.description.as_deref().unwrap_or("")
            }
            BookResult::OpenLibrary(_) => "",
        };

        // Count words in existing description
        let word_count = existing_description
            .split_whitespace()
            .count();

        if self.config.app.verbose {
            println!("Existing synopsis has {} words (minimum required: {})", 
                word_count, self.config.app.min_synopsis_words);
        }

        // Check if synopsis is too short or missing
        if word_count < self.config.app.min_synopsis_words {
            println!("Synopsis too short ({} words), generating enhanced synopsis with LLM...", word_count);

            // Get enhanced book information for synopsis generation
            let title = book.get_full_title();
            let author = book.get_all_authors();
            
            let enhanced_info = crate::web_search::enhance_book_info_with_search(
                &title,
                &author,
                existing_description,
            ).await;

            // Generate synopsis using LLM
            let llm_provider = crate::llm::LlmProvider::from_config(&self.config)?;
            let generated_synopsis = llm_provider.generate_synopsis(
                &enhanced_info, 
                self.config.app.target_synopsis_words
            ).await?;

            Ok(Some(generated_synopsis))
        } else {
            Ok(None)
        }
    }

    async fn create_baserow_entry(
        &self,
        book: &BookResult,
        selected_categories: &[String],
        synopsis: &str,
        available_categories: &[crate::baserow::Category],
        is_ebook: bool,
        cover_images: Vec<crate::baserow::CoverImage>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Preparing Baserow entry with collected data...");
        }

        // Extract book information
        let title = book.get_full_title();
        let author = book.get_all_authors();
        let isbn = match book {
            BookResult::Google(google_book) => google_book.get_isbn_13().or_else(|| google_book.get_isbn_10()),
            BookResult::OpenLibrary(ol_book) => ol_book.get_best_isbn(),
        };

        // Convert category names to IDs
        let category_ids = self.baserow_client.find_category_ids_by_names(selected_categories, available_categories);
        
        if category_ids.is_empty() {
            return Err("No valid category IDs found for selected categories".into());
        }

        // Create the media entry
        let entry = crate::baserow::MediaEntry {
            title,
            author,
            isbn,
            synopsis: synopsis.to_string(),
            category: category_ids,
            read: false, // Default to not read
            rating: 0, // Default rating (0 = unrated)
            media_type: Some(if is_ebook { 3021 } else { 3020 }), // 3021 = Ebook, 3020 = Physical
            location: vec![], // Empty - to be filled manually by user
            cover: cover_images,
        };

        // Create the entry in Baserow
        let created_entry = self.baserow_client.create_media_entry(entry).await?;
        
        Ok(created_entry.id)
    }

    fn show_preflight_confirmation(
        &self,
        book: &BookResult,
        selected_categories: &[String],
        synopsis: &str,
        is_ebook: bool,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        println!("\n==================================================");
        println!("               📖 CONFIRMATION SUMMARY");
        println!("==================================================");
        
        // Book details
        println!("Title:     {}", book.get_full_title());
        println!("Author:    {}", book.get_all_authors());
        
        // ISBN if available
        if let Some(isbn) = match book {
            BookResult::Google(google_book) => google_book.get_isbn_13().or_else(|| google_book.get_isbn_10()),
            BookResult::OpenLibrary(ol_book) => ol_book.get_best_isbn(),
        } {
            println!("ISBN:      {}", isbn);
        }
        
        // Media type
        println!("Type:      {}", if is_ebook { "📱 Ebook" } else { "📚 Physical Book" });
        
        // Categories
        println!("Categories: {}", selected_categories.join(", "));
        
        // Synopsis (truncated for display)
        let display_synopsis = if synopsis.len() > 300 {
            format!("{}...", &synopsis[..297])
        } else {
            synopsis.to_string()
        };
        println!("Synopsis:  {}", display_synopsis);
        
        println!("==================================================");
        
        // Get user confirmation
        use dialoguer::{theme::ColorfulTheme, Confirm};
        
        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Add this book to your library?")
            .default(false)
            .interact()?;
        
        Ok(confirmation)
    }

    fn get_cover_image_url(&self, book: &BookResult) -> Option<String> {
        match book {
            BookResult::Google(google_book) => {
                // Get the highest quality image available from Google Books
                google_book.volume_info.image_links.as_ref().and_then(|links| {
                    // Prefer large, then medium, then small, then thumbnail
                    let base_url = links.large.as_ref()
                        .or(links.medium.as_ref())
                        .or(links.small.as_ref())
                        .or(links.thumbnail.as_ref())?;
                    
                    // Clean and optimize the URL - keep zoom=1 as it's required!
                    let cleaned_url = base_url
                        .replace("http://", "https://")   // Ensure HTTPS
                        .replace("&edge=curl", "");      // Remove edge effects only
                    
                    if self.config.app.verbose {
                        println!("Original Google Books URL: {}", base_url);
                        println!("Cleaned URL: {}", cleaned_url);
                    }
                    
                    Some(cleaned_url)
                })
            }
            BookResult::OpenLibrary(ol_book) => {
                // Generate Open Library cover URL if we have an ISBN
                if let Some(isbn) = ol_book.get_best_isbn() {
                    let url = format!("https://covers.openlibrary.org/b/isbn/{}-L.jpg", isbn);
                    if self.config.app.verbose {
                        println!("Open Library cover URL: {}", url);
                    }
                    Some(url)
                } else {
                    None
                }
            }
        }
    }

    async fn handle_cover_image_upload(&self, book: &BookResult) -> Vec<crate::baserow::CoverImage> {
        // Try primary cover image URL
        if let Some(image_url) = self.get_cover_image_url(book) {
            if self.config.app.verbose {
                println!("Found cover image URL: {}", image_url);
            }
            
            // Try download + direct upload approach
            match self.download_and_upload_image(&image_url, "cover.jpg").await {
                Ok(upload_response) => {
                    return vec![crate::baserow::CoverImage {
                        name: upload_response.name,
                    }];
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to download/upload primary cover image: {}", e);
                    
                    // Try fallback for Google Books using Open Library if we have ISBN
                    if let BookResult::Google(google_book) = book {
                        if let Some(isbn) = google_book.get_isbn_13().or_else(|| google_book.get_isbn_10()) {
                            let fallback_url = format!("https://covers.openlibrary.org/b/isbn/{}-L.jpg", isbn);
                            if self.config.app.verbose {
                                println!("Trying Open Library fallback: {}", fallback_url);
                            }
                            
                            match self.download_and_upload_image(&fallback_url, "cover-fallback.jpg").await {
                                Ok(upload_response) => {
                                    println!("✅ Successfully uploaded cover using Open Library fallback");
                                    return vec![crate::baserow::CoverImage {
                                        name: upload_response.name,
                                    }];
                                }
                                Err(fallback_e) => {
                                    eprintln!("⚠️  Fallback download/upload also failed: {}", fallback_e);
                                }
                            }
                        }
                    }
                    
                    // Both primary and fallback failed
                    println!("\n==================================================");
                    println!("📝 IMPORTANT: Please manually upload the cover image");
                    println!("   Primary URL: {}", image_url);
                    if let BookResult::Google(google_book) = book {
                        if let Some(isbn) = google_book.get_isbn_13().or_else(|| google_book.get_isbn_10()) {
                            println!("   Fallback URL: https://covers.openlibrary.org/b/isbn/{}-L.jpg", isbn);
                        }
                    }
                    println!("==================================================\n");
                    return vec![];
                }
            }
        } else {
            println!("\n==================================================");
            println!("📝 IMPORTANT: No cover image found");
            println!("   Please manually upload a cover image to your book entry");
            println!("==================================================\n");
            vec![]
        }
    }

    async fn download_and_upload_image(&self, image_url: &str, filename: &str) -> Result<crate::baserow::FileUploadResponse, Box<dyn std::error::Error>> {
        if self.config.app.verbose {
            println!("Downloading image from: {}", image_url);
        }
        
        // Download the image
        let response = reqwest::get(image_url).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to download image: HTTP {}", response.status()).into());
        }
        
        let image_data = response.bytes().await?;
        
        if self.config.app.verbose {
            println!("Downloaded {} bytes, uploading to Baserow...", image_data.len());
        }
        
        // Upload directly to Baserow
        let upload_response = self.baserow_client.upload_file_direct(image_data.to_vec(), filename).await?;
        
        Ok(upload_response)
    }
}