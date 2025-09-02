use clap::{Parser, Subcommand};

mod config;
use config::Config;

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

fn main() {
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

    match &cli.command {
        Commands::Add { isbn, title, author } => {
            if let Some(isbn_value) = isbn {
                if config.app.verbose {
                    println!("Adding book by ISBN: {}", isbn_value);
                }
                add_book_by_isbn(isbn_value, &config);
            } else if let (Some(title_value), Some(author_value)) = (title, author) {
                if config.app.verbose {
                    println!("Adding book by title: '{}' and author: '{}'", title_value, author_value);
                }
                add_book_by_title_author(title_value, author_value, &config);
            } else {
                eprintln!("Error: Please provide either --isbn OR both --title and --author");
                std::process::exit(1);
            }
        }
    }
}

fn add_book_by_isbn(_isbn: &str, _config: &Config) {
    println!("ISBN lookup not yet implemented");
}

fn add_book_by_title_author(_title: &str, _author: &str, _config: &Config) {
    println!("Title/author search not yet implemented");
}