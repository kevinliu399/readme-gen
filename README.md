# readme-gen

A tool to automatically generate README files for your projects.

## Language

- Rust
- TOML

## Dependencies

- tokio (features: full, version: 1)
- walkdir (version: 2)
- serde_json (version: 1.0)
- dotenv (version: 0.15)
- serde (features: derive, version: 1.0)
- proc-macro2 (version: 1.0)
- toml (version: 0.8.20)
- reqwest (features: blocking, json, version: 0.12.15)
- syn (features: full, version: 2)
- quote (version: 1.0)

## Project Structure

```
readme-gen/
├── src/
│   ├── explorer.rs
│   ├── llm.rs
│   ├── main.rs
│   └── summarizer.rs
└── Cargo.toml
```

### File Descriptions

-   **`src/explorer.rs`**:
    -   Responsible for exploring the repository structure and extracting code context.
    -   **Functions:**
        -   `map_extension_to_language`: Maps file extensions to programming languages (private).
        -   `parse_rust_file`: Parses Rust files and extracts information (private).
        -   `invalid_path`: Checks if a path is invalid (private).
        -   `is_cargo_toml`: Checks if a file is a `Cargo.toml` file (private).
        -   `walk_repo`: Walks through the repository directory and collects code context (public).
    -   **Structs:** `CargoPackage`, `RepoCodeContext`, `FileInformation`, `CargoToml`, `FunctionMeta`
    -   **Enums:** `RepoError`

-   **`src/llm.rs`**:
    -   Handles communication with the Large Language Model (LLM) to generate the README content.
    -   **Functions:**
        -   `generate_md`: Generates the README content using the LLM (public).
        -   `load_summarizer`: Loads the summarizer from a file (private).
    -   **Structs:** `GeminiRequest`, `Candidate`, `ContentResponse`, `Part`, `Content`, `PartResponse`, `GeminiResponse`

-   **`src/main.rs`**:
    -   The main entry point of the application.
    -   **Functions:**
        -   `main`: The main function (private).

-   **`src/summarizer.rs`**:
    -   Builds the input string for the LLM summarization.
    -   **Functions:**
        -   `build_input`: Builds the input string for the LLM (public).
    -   **Structs:** `LLMInput`

## How to Run

1.  Clone the repository.
2.  Navigate to the project directory.
3.  Run `cargo build`.
4.  Run `cargo run`.

## Additional Information

This project uses the following environment variables:

*   `GEMINI_API_KEY`: The API key for the Gemini LLM.
