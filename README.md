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

## Implementation Plan

[] 1. **Project Setup**  
   Initialize Rust project with required dependencies (clap, reqwest, serde, tokio, image).

[] 2. **Configuration System**  
   Implement `.env`/`config.yaml` for API keys and endpoints (NFR-01).

[] 3. **CLI Structure**  
   Design `wcm add` command supporting `--isbn`, `--title`, and `--author`.

[] 4. **Google Books API**  
   Integrate primary book data fetching (FR-02).

[] 5. **Open Library API**  
   Implement fallback for book data (FR-02).

[] 6. **Ambiguous Search Handling**  
   Add interactive selection for >5 search results (FR-03).

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

1. Add `config.yaml` to repository
2. Implement core CLI structure
3. Integrate primary API (Google Books)

*Note: All LLM calls will strictly use existing Baserow categories (no new categories created).*
