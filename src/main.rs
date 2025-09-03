use clap::{Parser, Subcommand};

mod config;
mod google_books;
mod open_library;
mod book_search;
mod baserow;
mod web_search;
mod llm;
mod label;

use config::Config;
use google_books::GoogleBooksClient;
use open_library::OpenLibraryClient;
use book_search::CombinedBookSearcher;
use baserow::BaserowClient;
use label::LabelGenerator;

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
        
        #[arg(long, help = "Mark as ebook (default: physical book)")]
        ebook: bool,
    },
    Test {
        #[arg(long, help = "Test Baserow connection")]
        baserow: bool,
    },
    Label {
        #[arg(long, help = "Generate label by storage ID")]
        storage_id: Option<u64>,
        
        #[arg(long, help = "Generate label by storage name")]
        storage_name: Option<String>,
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

    // Create API clients
    let google_client = GoogleBooksClient::new(
        config.google_books.api_key.clone(),
        config.google_books.base_url.clone(),
    );
    let open_library_client = OpenLibraryClient::new(
        config.open_library.base_url.clone(),
    );
    let baserow_client = BaserowClient::new(config.baserow.clone());

    // Create combined searcher and label generator
    let searcher = CombinedBookSearcher::new(google_client, open_library_client, baserow_client.clone(), config.clone());
    let label_generator = LabelGenerator::new(baserow_client.clone(), config.baserow.base_url.clone());

    match &cli.command {
        Commands::Add { isbn, title, author, ebook } => {
            if let Some(isbn_value) = isbn {
                if config.app.verbose {
                    println!("Adding {} by ISBN: {}", if *ebook { "ebook" } else { "book" }, isbn_value);
                }
                if let Err(e) = add_book_by_isbn(isbn_value, &searcher, *ebook).await {
                    eprintln!("Error adding book by ISBN: {}", e);
                    std::process::exit(1);
                }
            } else if let (Some(title_value), Some(author_value)) = (title, author) {
                if config.app.verbose {
                    println!("Adding {} by title: '{}' and author: '{}'", if *ebook { "ebook" } else { "book" }, title_value, author_value);
                }
                if let Err(e) = add_book_by_title_author(title_value, author_value, &searcher, *ebook).await {
                    eprintln!("Error adding book by title/author: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Please provide either --isbn OR both --title and --author");
                std::process::exit(1);
            }
        }
        Commands::Test { baserow } => {
            if *baserow {
                println!("Testing Baserow connection...");
                if let Err(e) = baserow_client.test_connection().await {
                    eprintln!("Baserow connection test failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Label { storage_id, storage_name } => {
            if let Some(id) = storage_id {
                let filename = format!("storage_label_{}.png", id);
                let output_path = std::path::Path::new(&filename);
                if let Err(e) = label_generator.generate_label_by_id(*id, config.baserow.storage_table_id, config.baserow.database_id, config.baserow.storage_view_id, output_path).await {
                    eprintln!("Error generating label by ID: {}", e);
                    std::process::exit(1);
                }
            } else if let Some(name) = storage_name {
                let safe_name = name.replace(" ", "_").replace("/", "_");
                let filename = format!("storage_label_{}.png", safe_name);
                let output_path = std::path::Path::new(&filename);
                if let Err(e) = label_generator.generate_label_by_name(name, config.baserow.storage_table_id, config.baserow.database_id, config.baserow.storage_view_id, output_path).await {
                    eprintln!("Error generating label by name: {}", e);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Please provide either --storage-id OR --storage-name");
                std::process::exit(1);
            }
        }
    }
}

async fn add_book_by_isbn(
    isbn: &str,
    searcher: &CombinedBookSearcher,
    is_ebook: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    searcher.search_by_isbn(isbn, is_ebook).await?;
    Ok(())
}

async fn add_book_by_title_author(
    title: &str, 
    author: &str,
    searcher: &CombinedBookSearcher,
    is_ebook: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    searcher.search_by_title_author(title, author, is_ebook).await?;
    Ok(())
}

