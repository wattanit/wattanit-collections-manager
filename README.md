# Wattanit Collection Manager (wcm)

A CLI tool to automate adding books to your personal Baserow library using public APIs and LLMs.

## Features

- Add books via ISBN or title/author
- Fetch data from Google Books (primary) + Open Library (fallback)
- Interactive selection for ambiguous searches
- Auto-download and upload book covers
- LLM-powered category selection (from existing Baserow categories)
- Generate synopses when API data is insufficient
- Pre-flight confirmation before database writes

## Technical Stack

- **Language**: Rust
- **CLI**: `clap`
- **HTTP**: `reqwest`
- **Data Handling**: `serde`, `tokio`
- **Image Processing**: `image`
- **LLM Integration**: Modular (OpenAI/Claude/Ollama)

## Configuration

The application supports two methods for configuration:

### Method 1: YAML Configuration File

1. Copy the template configuration file:
   ```bash
   cp config.yaml my-config.yaml
   ```

2. Edit `my-config.yaml` and update the placeholder values:
   ```yaml
   # Required API keys
   google_books:
     api_key: "your_actual_google_books_api_key"
   
   baserow:
     api_token: "your_actual_baserow_token"
     database_id: 12345  # Your actual database ID
     media_table_id: 67890  # Your actual table ID
     categories_table_id: 11111  # Your actual categories table ID
   
   # LLM provider (choose one: openai, anthropic, ollama)
   llm:
     provider: "ollama"  # Default provider
     
     # OpenAI Configuration
     openai:
       api_key: "your_actual_openai_key"
       model: "gpt-4"
     
     # Anthropic Configuration  
     anthropic:
       api_key: "your_actual_anthropic_key"
       model: "claude-3-sonnet-20240229"
     
     # Ollama Configuration (local - no API key needed)
     ollama:
       model: "gpt-oss:20b"
   ```

3. Rename the file to `config.yaml` or set the config file path.

### Method 2: Environment Variables

1. Copy the environment template:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your actual values:
   ```bash
   GOOGLE_BOOKS_API_KEY=your_actual_google_books_api_key
   BASEROW_API_TOKEN=your_actual_baserow_token
   BASEROW_DATABASE_ID=12345
   BASEROW_MEDIA_TABLE_ID=67890
   BASEROW_CATEGORIES_TABLE_ID=11111
   OPENAI_API_KEY=your_actual_openai_key
   ```

### Configuration Priority

Environment variables override YAML configuration values. This allows you to:
- Use YAML for base configuration
- Override specific values with environment variables for different environments
- Use only environment variables in production/CI environments

### Required API Keys

- **Google Books API**: Optional - works without API key for basic usage. Get key from [Google Cloud Console](https://console.cloud.google.com/) for higher rate limits
- **Baserow API**: Generate a token in your Baserow account settings
- **LLM Provider**: Choose one:
  - OpenAI: API key from [OpenAI Platform](https://platform.openai.com/)
  - Anthropic: API key from [Anthropic Console](https://console.anthropic.com/)
  - Ollama: No API key needed (runs locally)

## Implementation Plan

[✓] 1. **Project Setup**  
   Initialize Rust project with required dependencies (clap, reqwest, serde, tokio, image).

[✓] 2. **Configuration System**  
   Implement `.env`/`config.yaml` for API keys and endpoints (NFR-01).

[✓] 3. **CLI Structure**  
   Design `wcm add` command supporting `--isbn`, `--title`, and `--author`.

[✓] 4. **Google Books API**  
   Integrate primary book data fetching (FR-02).

[✓] 5. **Open Library API**  
   Implement fallback for book data (FR-02).

[✓] 6. **Ambiguous Search Handling**  
   Add interactive selection for ambiguous search results (FR-03).

[] 7. **Cover Image Handling**  
   Download highest-res cover + upload to Baserow (FR-04).

[] 8. **Category Pre-fetch**  
   Fetch existing categories from Baserow `Categories` table (FR-05).

[] 9. **LLM Category Prompt**  
   Craft prompt enforcing 3-5 existing categories only (FR-05).

[] 10. **LLM Category Selection**  
    Implement LLM category selection using prompt (FR-05).

[] 11. **Synopsis Generation**  
    Check API synopsis length; trigger LLM if <50 words (FR-06).

[] 12. **LLM Synopsis Generation**  
    Implement LLM synopsis (150 words, spoiler-free) (FR-06).

[] 13. **Baserow Entry Creation**  
    Map all data to Baserow fields via API (FR-07).

[] 14. **Pre-flight Confirmation**  
    Add summary + `[y/N]` prompt before database write (FR-08).

[] 15. **User Feedback**  
    Add step-by-step logging (e.g., "Fetching data...") (NFR-03).

[] 16. **Multi-LLM Architecture**  
    Design modular LLM interface for OpenAI/Claude/Ollama (NFR-02).

## Next Steps

1. Implement cover image downloading and processing
2. Implement Baserow category pre-fetching
3. Add LLM integration for category selection

*Note: All LLM calls will strictly use existing Baserow categories (no new categories created).*
