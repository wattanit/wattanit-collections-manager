use clap::{Parser, Subcommand};

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

    match &cli.command {
        Commands::Add { isbn, title, author } => {
            if let Some(isbn_value) = isbn {
                println!("Adding book by ISBN: {}", isbn_value);
            } else if let (Some(title_value), Some(author_value)) = (title, author) {
                println!("Adding book by title: '{}' and author: '{}'", title_value, author_value);
            } else {
                eprintln!("Error: Please provide either --isbn OR both --title and --author");
                std::process::exit(1);
            }
        }
    }
}