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
wcm add --isbn 9780345391803                                    # Add book by ISBN
wcm add --title "The Lord of the Rings" --author "J.R.R. Tolkien"  # Add book by title/author
```

## Architecture Overview

The project follows a modular architecture with these key components:

### Core Modules
- **CLI Interface**: Uses `clap` for command parsing and user interaction
- **Book Data Sources**: Google Books API (primary) + Open Library API (fallback)
- **LLM Integration**: Modular interface supporting OpenAI/Claude/Ollama for:
  - Category selection from existing Baserow categories (3-5 selections)
  - Synopsis generation (150 words, spoiler-free) when API data is insufficient
- **Image Processing**: Downloads highest-resolution covers using `image` crate
- **Baserow Integration**: Creates database entries via REST API

### Key Dependencies
- `clap` - CLI argument parsing
- `reqwest` - HTTP client for API calls
- `serde` - JSON/YAML serialization
- `tokio` - Async runtime
- `image` - Cover image processing
- `dialoguer` - Interactive terminal selection menus

### Data Flow
1. Parse CLI input (ISBN or title/author)
2. Fetch book data from APIs
3. Handle ambiguous searches with interactive selection
4. Fetch existing categories from Baserow
5. Use LLM for category selection (from existing categories only)
6. Generate synopsis via LLM if needed
7. Display pre-flight confirmation
8. Create Baserow entry
9. Download and process cover image (final enhancement)

## Important Implementation Notes

- **Configuration**: Uses `config.yaml` or `.env` for API keys and endpoints
- **Category Constraint**: LLMs must only select from existing Baserow categories, never create new ones
- **Fallback Logic**: Google Books API is primary, Open Library is fallback
- **User Experience**: Provides step-by-step feedback and requires confirmation before database writes
- **Interactive Selection**: Handles ambiguous search results with arrow-key selection menus (limited by max_search_results)

## Current Status

Complete book search and Baserow integration implemented:
- Project setup with all required dependencies including dialoguer for interactive selection
- Complete configuration system with YAML and environment variable support
- CLI structure with clap for `wcm add` command supporting ISBN and title/author inputs
- Google Books API integration with comprehensive book data fetching (works without API key)
- Open Library API integration with intelligent fallback system
- Interactive selection menus for ambiguous search results with proper truncation logic
- Rich book information display from both APIs with source identification
- Smart max_search_results limiting to prevent overwhelming users
- Baserow client with category pre-fetching functionality
- Successfully connects to self-hosted Baserow instance and fetches existing categories

The application now provides a complete dual-API book search experience with working Baserow integration. Category fetching is operational, retrieving all 69 categories from the Categories table. Next steps focus on LLM functionality for automated category selection and synopsis generation according to the 16-step plan in README.md.
- Do not use emoji, especially when writing documentation