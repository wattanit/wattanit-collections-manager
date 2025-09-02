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

### Data Flow
1. Parse CLI input (ISBN or title/author)
2. Fetch book data from APIs
3. Handle ambiguous searches with interactive selection
4. Download and process cover image
5. Fetch existing categories from Baserow
6. Use LLM for category selection (from existing categories only)
7. Generate synopsis via LLM if needed
8. Display pre-flight confirmation
9. Create Baserow entry

## Important Implementation Notes

- **Configuration**: Uses `config.yaml` or `.env` for API keys and endpoints
- **Category Constraint**: LLMs must only select from existing Baserow categories, never create new ones
- **Fallback Logic**: Google Books API is primary, Open Library is fallback
- **User Experience**: Provides step-by-step feedback and requires confirmation before database writes
- **Error Handling**: Handles ambiguous search results with interactive selection (top 5 results)

## Current Status

Basic project foundation is complete:
- Project setup with all required dependencies (Cargo.toml)
- CLI structure implemented with clap for `wcm add` command supporting ISBN and title/author inputs
- Basic argument validation and error handling in place

Next steps focus on configuration system and API integrations according to the 16-step implementation plan in README.md.
- Do not use emoji, especially when writing documentation