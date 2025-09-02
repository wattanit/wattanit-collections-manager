# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Wattanit Collection Manager (wcm) is a Rust CLI tool that automates adding books to a personal Baserow library using public APIs and LLMs. The tool fetches book data from Google Books API (primary) and Open Library API (fallback), downloads cover images, uses LLMs for category selection and synopsis generation, then creates entries in Baserow.

## Key Commands

### Build and Development
```bash
cargo build          # Build the project
cargo run             # Run the CLI tool
cargo test            # Run tests
cargo check           # Check for compilation errors without building
cargo clippy          # Lint with clippy
cargo fmt             # Format code
```

### CLI Usage
```bash
wcm add --isbn 9780345391803                                    # Add physical book by ISBN
wcm add --isbn 9780345391803 --ebook                           # Add ebook by ISBN
wcm add --title "The Lord of the Rings" --author "J.R.R. Tolkien"  # Add book by title/author
wcm test --baserow                                              # Test Baserow connection
```

## Architecture Overview

The project follows a modular architecture with these key components:

### Core Modules
- **CLI Interface**: Uses `clap` for command parsing and user interaction with ebook/physical classification
- **Book Data Sources**: Google Books API (primary) + Open Library API (fallback)
- **Web Search Enhancement**: DuckDuckGo API integration for comprehensive book information
- **LLM Integration**: Modular interface supporting OpenAI/Claude/Ollama for:
  - Category selection from existing Baserow categories (3-5 selections)
  - Synopsis generation (150 words, spoiler-free) when API data is insufficient
- **Image Processing**: Downloads highest-resolution covers using `image` crate (pending)
- **Baserow Integration**: Complete database entry creation via REST API with media type classification

### Key Dependencies
- `clap` - CLI argument parsing
- `reqwest` - HTTP client for API calls
- `serde` - JSON/YAML serialization
- `tokio` - Async runtime
- `image` - Cover image processing
- `dialoguer` - Interactive terminal selection menus

### Data Flow
1. Parse CLI input (ISBN or title/author, optional ebook flag)
2. Fetch book data from APIs with web search enhancement if needed
3. Handle ambiguous searches with interactive selection
4. Fetch existing categories from Baserow
5. Use LLM for category selection (from existing categories only)
6. Generate synopsis via LLM if needed
7. Create Baserow entry with appropriate media type (ebook/physical)
8. Display pre-flight confirmation (pending)
9. Download and process cover image (pending)

## Important Implementation Notes

- **Configuration**: Uses `config.yaml` or `.env` for API keys and endpoints
- **Category Constraint**: LLMs must only select from existing Baserow categories, never create new ones
- **Fallback Logic**: Google Books API is primary, Open Library is fallback
- **User Experience**: Provides step-by-step feedback and requires confirmation before database writes
- **Interactive Selection**: Handles ambiguous search results with arrow-key selection menus (limited by max_search_results)

## Current Status

Complete end-to-end book processing pipeline with full Baserow integration:
- Project setup with all required dependencies including dialoguer for interactive selection
- Complete configuration system with YAML and environment variable support
- CLI structure with clap for `wcm add` command supporting ISBN, title/author, and ebook classification
- Google Books API integration with comprehensive book data fetching (works without API key)
- Open Library API integration with intelligent fallback system
- Interactive selection menus for ambiguous search results with proper truncation logic
- Rich book information display from both APIs with enhanced description viewing (1000 chars)
- Smart max_search_results limiting to prevent overwhelming users
- Baserow client with complete database integration
- Successfully connects to self-hosted Baserow instance and manages 69+ categories
- Web search integration using DuckDuckGo API for enhanced book information gathering
- Complete LLM integration with modular architecture supporting OpenAI/Claude/Ollama
- Intelligent category selection using LLM with existing Baserow categories constraint
- Automatic synopsis generation when existing descriptions are insufficient (50+ word threshold)
- Full database entry creation with proper field mapping and media type classification
- Working ebook vs physical book classification via --ebook CLI flag

The application now provides a fully functional book management system from search through database entry creation. All core functionality is implemented and tested. Remaining work includes pre-flight confirmation and cover image handling enhancements.
- Do not use emoji, especially when writing documentation