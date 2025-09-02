use clap::{Parser, Subcommand};

mod config;
mod google_books;

use config::Config;
use google_books::GoogleBooksClient;

#[derive(Parser)]
#[command(name = "wcm")]
#[command(about = "Wattanit Collection Manager - A CLI tool to automate adding books to your personal Baserow library")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[arg(long, help = "Add book by ISBN")]
        isbn: Option<String>,
        
        #[arg(long, help = "Book title")]
        title: Option<String>,
        
        #[arg(long, help = "Book author")]
        author: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            eprintln!("Make sure config.yaml exists or required environment variables are set.");
            std::process::exit(1);
        }
    };
    
    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        eprintln!("Please check your config.yaml or .env file.");
        std::process::exit(1);
    }
    
    if config.app.verbose {
        println!("Configuration loaded successfully");
        println!("LLM Provider: {}", config.llm.provider);
    }

    // Create Google Books client
    let google_client = GoogleBooksClient::new(
        config.google_books.api_key.clone(),
        config.google_books.base_url.clone(),
    );

    match &cli.command {
        Commands::Add { isbn, title, author } => {
            if let Some(isbn_value) = isbn {
                if config.app.verbose {
                    println!("Adding book by ISBN: {}", isbn_value);
                }
                if let Err(e) = add_book_by_isbn(isbn_value, &config, &google_client).await {
                    eprintln!("Error adding book by ISBN: {}", e);
                    std::process::exit(1);
                }
            } else if let (Some(title_value), Some(author_value)) = (title, author) {
                if config.app.verbose {
                    println!("Adding book by title: '{}' and author: '{}'", title_value, author_value);
                }
                if let Err(e) = add_book_by_title_author(title_value, author_value, &config, &google_client).await {
                    eprintln!("Error adding book by title/author: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Please provide either --isbn OR both --title and --author");
                std::process::exit(1);
            }
        }
    }
}

async fn add_book_by_isbn(
    isbn: &str, 
    config: &Config, 
    google_client: &GoogleBooksClient
) -> Result<(), Box<dyn std::error::Error>> {
    if config.app.verbose {
        println!("Fetching book data from Google Books API...");
    }
    
    let response = google_client.search_by_isbn(isbn).await?;
    
    if response.total_items == 0 {
        println!("No books found for ISBN: {}", isbn);
        return Ok(());
    }
    
    if let Some(items) = &response.items {
        if let Some(book) = items.first() {
            display_book_info(book, config);
        }
    }
    
    Ok(())
}

async fn add_book_by_title_author(
    title: &str, 
    author: &str, 
    config: &Config, 
    google_client: &GoogleBooksClient
) -> Result<(), Box<dyn std::error::Error>> {
    if config.app.verbose {
        println!("Searching for books on Google Books API...");
    }
    
    let response = google_client.search_by_title_author(title, author).await?;
    
    if response.total_items == 0 {
        println!("No books found for title: '{}' and author: '{}'", title, author);
        return Ok(());
    }
    
    if let Some(items) = &response.items {
        if items.len() > 1 && items.len() <= config.app.max_search_results {
            println!("Found {} books. Please select one:", items.len());
            for (i, book) in items.iter().enumerate() {
                println!("{}. {} by {}", 
                    i + 1, 
                    book.get_full_title(), 
                    book.get_all_authors()
                );
            }
            println!("Interactive selection not yet implemented - showing first result:");
        }
        
        if let Some(book) = items.first() {
            display_book_info(book, config);
        }
    }
    
    Ok(())
}

fn display_book_info(book: &google_books::BookItem, _config: &Config) {
    println!("\n=== Book Information ===");
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
    
    println!("========================\n");
}