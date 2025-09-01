# **Requirement Document: `wcm` - Wattanit Collection Manager CLI**

* **Version:** 1.2
* **Date:** 2 September 2025
* **Author:** Gemini & Wattanit
* **Project:** Personal Library Cataloging Automation Tool

***

## **1. Introduction**

**`wcm`** is a command-line interface designed to automate and enrich the process of adding books to a personal Baserow database. The tool will reduce the friction of manual data entry by sourcing information from public APIs and leveraging Large Language Models (LLMs) for intelligent data generation, ensuring a rich, consistent, and well-organized library.

***

## **2. Core Functional Requirements**

**FR-01: Add Command with Multiple Input Modes**
The CLI's primary function will be `wcm add`, which must support two methods for adding a book:
* **By ISBN:** `wcm add --isbn 9780345391803`
* **By Title and Author:** `wcm add --title "The Lord of the Rings" --author "J.R.R. Tolkien"`

---
**FR-02: Book Data Sourcing**
The application must fetch structured book data from established online sources.
* **Primary Source:** Google Books API.
* **Fallback Source:** Open Library API.

---
**FR-03: Interactive Selection for Ambiguous Searches**
If a title/author search returns multiple potential matches, the CLI must present the user with a numbered list of the top 5 results and prompt for a selection.

---
**FR-04: Cover Image Handling**
The CLI must download the highest-resolution cover image available and upload the file **directly to the Baserow file field** via the API.

---
**FR-05 (Revised): LLM-Powered Category Selection from a Pre-defined List**
* **Pre-computation Step:** Before making any LLM calls, the `wcm` CLI must first perform a `GET` request to the Baserow API to fetch all existing entries from the `Categories` table.
* **Trigger:** This function will run for every new book entry.
* **Input to LLM:** The book's title and description, **along with the complete list of available categories fetched from Baserow.**
* **LLM Prompt:** The prompt should instruct the LLM to act as an expert librarian and select the 3-5 most relevant categories for the book **exclusively from the provided list**. The prompt must explicitly forbid the LLM from creating new categories.
* **Output:** The LLM's selection will be used to populate the "Multiple Select" (or linked) category field in the `Media` table.

---
**FR-06: LLM-Powered Synopsis Generation**
The CLI will use an LLM to write a compelling, spoiler-free synopsis of approximately 150 words **only if** the API-provided synopsis is missing or inadequate (e.g., less than 50 words).

---
**FR-07: Baserow Entry Creation**
The CLI will create a new row in the specified Baserow table, correctly mapping all collected data to the corresponding fields.

---
**FR-08: Pre-flight Confirmation**
Before writing to the database, the CLI will display a summary of all collected and generated data and ask the user for a final confirmation `[y/N]`.

***

## **3. Non-Functional Requirements**

**NFR-01: Configuration Management**
All configuration (API keys, URLs, etc.) must be managed via a `config.yaml` or `.env` file.

---
**NFR-02: Multi-Provider LLM Support**
The application architecture must be modular to support **OpenAI**, **Anthropic Claude**, and **Ollama**.

---
**NFR-03: User Feedback and Logging**
The CLI must provide clear, step-by-step feedback to the user during its operation (e.g., "Fetching data...", "Generating categories...").

***

## **4. Technical Specifications**

* **Primary Language:** **Rust** 🦀
* **Key Crates (Libraries):**
    * **CLI Parsing:** `clap`
    * **HTTP Client:** `reqwest`
    * **JSON/YAML Handling:** `serde`
    * **Async Runtime:** `tokio`
    * **Image Handling:** `image`
* **Baserow API Endpoint for Row Creation:** `POST /api/database/rows/table/{table_id}/?user_field_names=true`