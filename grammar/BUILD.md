# Grammar Build Process

The `grammar/build.rs` script automates the process of integrating Tree-sitter grammars from the Helix editor project into this codebase. It ensures that syntax highlighting and parsing capabilities are kept up-to-date and correctly configured.

## Key Functionality:

1.  **Fetching Helix Grammars**: The script clones the official `helix-editor/helix` GitHub repository at a specific commit reference (`GRAMMAR_REPOSITORY_REF`). This provides access to the latest Tree-sitter grammars and language configurations.

2.  **Language Configuration Parsing**: It reads and parses the `languages.toml` file from the fetched Helix repository. This file defines the various programming languages supported by Helix, their associated Tree-sitter grammars, file type associations (extensions, globs), and injection regexes.

3.  **Grammar Compilation**: For each active grammar (excluding those in `BLACKLISTED_MODULES`), the `build.rs` script:
    *   Clones individual grammar repositories if their source is specified as Git.
    *   Compiles the C/C++ source files (`parser.c`, `parser.cc`, `scanner.c`, `scanner.cc`) for each grammar using the `cc` Rust crate. This generates the necessary shared library files for Tree-sitter.

4.  **Rust Module Generation**: The script dynamically generates several Rust modules (`.rs` files) in the `OUT_DIR` (typically `target/debug/build/gnostr-xxxx/out` or `target/release/...`):
    *   `grammar.defs.rs`: Contains `pub mod` definitions for each grammar, exposing its `LANGUAGE` function (Tree-sitter's entry point) and constants for `HIGHLIGHTS_QUERY`, `INJECTIONS_QUERY`, and `LOCALS_QUERY` strings. These query strings are used by Tree-sitter for syntax highlighting, code injection, and local variable detection.
    *   `grammar.registry.rs`: Defines a `Grammar` enum, providing a structured way to reference all available grammars. It also implements methods to retrieve highlight configuration parameters (`HighlightConfigurationParams`) and an index for each grammar variant.
    *   `language.registry.rs`: Defines a `Language` enum, which represents each supported programming language. It includes methods (`from_file_name`, `from_injection`) to identify a language based on file extensions, glob patterns, or injection regexes, leveraging the `globset` and `regex` crates for efficient matching.

5.  **Query Inheritance Handling**: The `read_local_query` function is designed to process Tree-sitter query files (`.scm`). It supports an `inherits` directive, allowing grammars to inherit and compose queries from other grammars, reducing duplication and promoting modularity in query definitions.

## Dependencies:

*   **`git`**: Used for cloning the Helix repository and individual grammar repositories.
*   **`cc` crate**: A Rust build dependency used to compile C/C++ source files for the Tree-sitter parsers.
*   **`toml` crate**: For parsing the `languages.toml` configuration file.
*   **`quote` and `proc_macro2` crates**: Used for programmatic Rust code generation.
*   **`regex` and `globset` crates**: For efficient pattern matching when identifying languages.
*   **`prettyplease` crate**: Used for formatting the generated Rust code.
*   **`threadpool` crate**: For parallelizing grammar fetching and compilation.

## How it uses Helix:

The script directly leverages the `helix-editor/helix` repository as the primary source for its Tree-sitter grammars and language configuration. It adheres to Helix's `languages.toml` structure and its query file conventions (`highlights.scm`, `injections.scm`, `locals.scm`) to ensure compatibility and consistent syntax support. By fetching grammars from Helix, this project benefits from the continuous development and refinement of syntax definitions maintained by the Helix community.
