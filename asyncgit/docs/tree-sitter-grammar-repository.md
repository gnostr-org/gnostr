# Tree-Sitter Grammar Repository in Gnostr-Gnit

## Overview

The **tree-sitter-grammar-repository** is a custom Rust crate that provides comprehensive syntax highlighting capabilities for the gnostr-gnit git hosting application. It leverages Tree-sitter, a modern incremental parsing system, to offer high-performance syntax highlighting for dozens of programming languages.

This repository is based on Helix editor's grammar definitions and provides both static and dynamic linking options for language grammars, making it highly flexible for different deployment scenarios.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Gnostr-Gnit Application                  │
├─────────────────────────────────────────────────────────────┤
│  syntax_highlight.rs                                        │
│  ├── ComrakHighlightAdapter                                │
│  ├── Highlighter Engine                                    │
│  └── Theme System                                          │
├─────────────────────────────────────────────────────────────┤
│  tree-sitter-grammar-repository                            │
│  ├── Grammar Registry                                      │
│  ├── Language Detection                                    │
│  └── Configuration Management                              │
├─────────────────────────────────────────────────────────────┤
│  Tree-Sitter Core Libraries                                │
│  ├── tree-sitter-highlight                                 │
│  ├── tree-sitter-language                                  │
│  └── Language Parsers                                     │
├─────────────────────────────────────────────────────────────┤
│  Grammar Sources                                           │
│  ├── Helix Editor Repository (Static)                      │
│  └── Dynamic Library Directory (Optional)                  │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

```
File Request → Language Detection → Grammar Selection → Parsing → Highlighting → HTML Output
     ↓                ↓                   ↓            ↓           ↓            ↓
file.rs          .rs extension        Rust grammar   Tree-sitter   CSS classes   <span class="highlight keyword">
```

## Core Components

### 1. Grammar Repository Crate (`tree-sitter-grammar-repository/`)

**Location**: `tree-sitter-grammar-repository/src/lib.rs`

The grammar repository is the heart of the syntax highlighting system:

```rust
// Main grammar registry
pub enum Grammar {
    Rust,
    JavaScript,
    Python,
    TypeScript,
    // ... 50+ more languages
}

impl Grammar {
    pub const fn highlight_configuration_params(self) -> HighlightConfigurationParams {
        // Returns language function, name, and query strings
    }
}

// Language detection based on file extensions and glob patterns
pub enum Language {
    Rust,
    JavaScript,
    // ... corresponding language variants
}

impl Language {
    pub fn from_file_name<P: AsRef<Path>>(name: P) -> Option<Self> {
        // Intelligently detects language from file path
    }

    pub fn from_injection(name: &str) -> Option<Self> {
        // Handles embedded language detection
    }
}
```

### 2. Build System (`tree-sitter-grammar-repository/build.rs`)

The build system handles grammar compilation and registry generation:

- **Static Mode**: Downloads and compiles grammars from Helix repository
- **Dynamic Mode**: Links against pre-compiled shared libraries
- **Registry Generation**: Creates efficient lookup tables at compile time

### 3. Syntax Highlighting Engine (`src/syntax_highlight.rs`)

**Location**: `src/syntax_highlight.rs:35`

The highlighting engine provides the core syntax highlighting functionality:

```rust
pub struct ComrakHighlightAdapter;

impl SyntaxHighlighterAdapter for ComrakHighlightAdapter {
    fn write_highlighted(&self, output: &mut dyn IoWrite,
                        lang: Option<&str>, code: &str) -> std::io::Result<()> {
        // Applies syntax highlighting to code blocks
    }
}

// Core highlighting function
pub fn format_file(content: &str, identifier: FileIdentifier<'_>) -> anyhow::Result<String> {
    // Processes file content with appropriate grammar
}
```

## Integration with Git Hosting

### 1. Web Interface Integration

**Location**: `src/git.rs:35`

The syntax highlighting integrates with the git web interface:

```rust
use crate::syntax_highlight::{
    format_file, format_file_inner,
    ComrakHighlightAdapter, FileIdentifier
};

// Markdown rendering with syntax highlighting
plugins.render.codefence_syntax_highlighter = Some(&ComrakHighlightAdapter);
```

### 2. File Tree Rendering

**Location**: `src/methods/repo/tree.rs`

When displaying file contents in the web interface:

```rust
// File identifier creation for syntax detection
let identifier = FileIdentifier::Path(&file_path);
let highlighted_content = format_file(&file_content, identifier)?;
```

## Configuration Options

### Environment Variables

| Variable                      | Purpose                     | Default | Example                      |
| ----------------------------- | --------------------------- | ------- | ---------------------------- |
| `TREE_SITTER_GRAMMAR_LIB_DIR` | Enable dynamic linking mode | Not set | `/usr/local/lib/tree-sitter` |

### Directory Structure for Dynamic Mode

When `TREE_SITTER_GRAMMAR_LIB_DIR` is set:

```
TREE_SITTER_GRAMMAR_LIB_DIR/
├── languages.toml              # Language definitions
├── queries/                    # Query files for each language
│   ├── rust/
│   │   ├── highlights.scm
│   │   ├── injections.scm
│   │   └── locals.scm
│   ├── javascript/
│   │   ├── highlights.scm
│   │   ├── injections.scm
│   │   └── locals.scm
│   └── ...
├── libtree-sitter-rust.so      # Compiled grammar libraries
├── libtree-sitter-javascript.so
└── ...
```

## Build Process and Grammar Compilation

### Static Build Process

1. **Repository Fetching**: Downloads Helix editor repository

   ```bash
   # From build.rs:70
   git clone --depth 1 https://github.com/helix-editor/helix
   git checkout 82dd96369302f60a9c83a2d54d021458f82bcd36
   ```

2. **Grammar Compilation**: Compiles each grammar using C/C++ compilers

   ```rust
   // From build.rs:408
   cc::Build::new()
       .cpp(scanner_file.extension() == Some(OsStr::new("cc")))
       .file(parser_file)
       .include(&grammar_src)
       .compile(&format!("{}-parser", grammar.name));
   ```

3. **Query Processing**: Processes Tree-sitter query files

   ```rust
   // From build.rs:332
   fn read_local_query(query_path: &Path, language: &str, filename: &str) -> String {
       // Handles query inheritance and processing
   }
   ```

4. **Registry Generation**: Creates efficient lookup tables
   - Grammar registry for parser access
   - Language registry for file type detection
   - Injection regex for embedded languages

### Supported Languages

The system supports **50+ programming languages** including:

- **Popular Languages**: Rust, JavaScript, Python, TypeScript, Go, Java
- **Web Technologies**: HTML, CSS, SCSS, JSX, TSX
- **Configuration**: JSON, TOML, YAML, INI
- **Systems Programming**: C, C++, Zig, Nim
- **Specialized**: Dockerfile, SQL, Makefile, Regex
- **And many more...**

### Blacklisted Languages

Some languages are excluded due to compilation issues:

```rust
static BLACKLISTED_MODULES: &[&str] = &[
    "cabal", "idris", "llvm-mir-yaml", "prolog",
    "mint", "hare", "wren", "gemini", // macOS compilation issues
];
```

## Performance Considerations

### 1. Lazy Initialization

Grammar configurations are loaded on-demand:

```rust
static HIGHLIGHTER_CONFIGS: LazyLock<Vec<HighlightConfiguration>> = LazyLock::new(|| {
    Grammar::VARIANTS
        .iter()
        .copied()
        .map(Grammar::highlight_configuration_params)
        .map(|v| HighlightConfiguration::new(...).configure(&HIGHLIGHT_NAMES))
        .collect()
});
```

### 2. Thread-Local Highlighter

Uses thread-local storage for the highlighter instance:

```rust
thread_local! {
    static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
}
```

### 3. Caching

File content and highlighter results are cached using moka:

```rust
pub struct Git {
    readme_cache: Cache<ReadmeCacheKey, Option<(ReadmeFormat, Arc<str>)>>,
    // ... other caches
}
```

### 4. Optimized Language Detection

Language detection uses glob patterns with priority sorting:

```rust
// Prefer more specific patterns over general ones
if curr_length < max {
    max = curr_length;
    curr = Some(GLOB_TO_VARIANT[m]);
}
```

## Theme System

### Theme Configuration

**Location**: `src/theme.rs`

Themes define visual appearance for syntax highlighting:

```toml
# Example: themes/onedark.toml
"keyword" = { fg = "red" }
"string" = { fg = "green" }
"comment" = { fg = "light-gray", modifiers = ["italic"] }
"function" = { fg = "blue" }

[palette]
red = "#E06C75"
green = "#98C379"
blue = "#61AFEF"
```

### CSS Generation

Themes are converted to CSS for web display:

```rust
impl Theme {
    pub fn build_css(&self) -> String {
        // Generates CSS classes like:
        // .highlight.keyword { color:#E06C75; }
        // .highlight.string { color:#98C379; }
    }
}
```

### Highlight Classes

**Location**: `src/syntax_highlight.rs:37-89`

Comprehensive set of highlight classes:

```rust
define_classes! {
    "attribute" => "attribute",
    "boolean" => "boolean",
    "comment" => "comment",
    "function" => "function",
    "keyword" => "keyword",
    "string" => "string",
    // ... 50+ more categories
}
```

## Adding New Languages

### Method 1: Static Integration

1. **Add to Helix Repository**: Contribute the language to Helix editor
2. **Update Build Script**: No changes needed - automatically detected
3. **Rebuild**: The build system will fetch and compile the new grammar

### Method 2: Custom Grammar

1. **Create Grammar Module**: In `tree-sitter-grammar-repository/src/`
2. **Define Language**: Add to `Language` enum in build script
3. **Add Queries**: Create query files in appropriate directory
4. **Build**: Compile and test the new grammar

### Example: Adding a Custom Language

```rust
// In build.rs
#[derive(Deserialize)]
struct LanguageDefinition {
    name: String,
    injection_regex: Option<String>,
    file_types: Vec<FileType>,
    grammar: Option<String>,
}

// Your custom language
{
    name: "mylang".to_string(),
    file_types: vec![
        FileType::Extension("ml".to_string()),
        FileType::Glob { glob: "Dockerfile.*".to_string() },
    ],
    grammar: Some("mylang".to_string()),
}
```

## Troubleshooting and Common Issues

### 1. Build Failures

**Issue**: Grammar compilation fails

**Solution**: Check if the language is in `BLACKLISTED_MODULES` and consider adding it:

```rust
static BLACKLISTED_MODULES: &[&str] = &[
    // Add problematic languages here
    "problematic-language",
];
```

### 2. Language Detection Issues

**Issue**: Wrong language detected for file

**Solution**: Check file type patterns in Helix's `languages.toml`:

```toml
[[language]]
name = "rust"
file-types = ["rs"]
```

### 3. Performance Issues

**Issue**: Slow highlighting performance

**Solutions**:

- Enable dynamic linking with `TREE_SITTER_GRAMMAR_LIB_DIR`
- Check for memory leaks in the highlighting pipeline
- Consider reducing the number of loaded grammars

### 4. Theme Rendering Issues

**Issue**: Colors not appearing correctly

**Solution**: Verify CSS class names match highlight names:

```rust
// Check this matches your theme
"keyword" => "keyword"  // Must match exactly
```

### 5. Dynamic Linking Issues

**Issue**: Shared libraries not found

**Solution**: Ensure library paths are correct:

```bash
export LD_LIBRARY_PATH=/path/to/grammars:$LD_LIBRARY_PATH
```

## Code Examples

### Basic Usage

```rust
use tree_sitter_grammar_repository::{Grammar, Language};
use crate::syntax_highlight::format_file;

// Detect language from file path
let language = Language::from_file_name("src/main.rs")?;
let grammar = language.grammar();

// Highlight code
let highlighted = format_file(
    "fn main() { println!(\"Hello\"); }",
    FileIdentifier::Path(Path::new("main.rs"))
)?;
```

### Markdown Integration

````rust
use comrak::{markdown_to_html, Options};
use crate::syntax_highlight::ComrakHighlightAdapter;

let mut options = Options::default();
let mut plugins = comrak::ComrakPlugins::default();
plugins.render.codefence_syntax_highlighter = Some(&ComrakHighlightAdapter);

let html = markdown_to_html("```rust\nfn main() {}\n```", &options, &plugins);
````

### Custom Theme

```rust
use crate::theme::Theme;
use std::collections::HashMap;

let theme = Theme {
    palette: {
        let mut p = HashMap::new();
        p.insert("blue".to_string(), "#61AFEF".to_string());
        p.insert("red".to_string(), "#E06C75".to_string());
        p
    },
    definitions: {
        let mut d = HashMap::new();
        d.insert("keyword".to_string(),
                PaletteReference::Foreground("red".to_string()));
        d.insert("function".to_string(),
                PaletteReference::Foreground("blue".to_string()));
        d
    }
};

let css = theme.build_css();
```

## Testing and Validation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_file_name("test.rs"), Some(Language::Rust));
        assert_eq!(Language::from_file_name("script.py"), Some(Language::Python));
    }

    #[test]
    fn test_highlighting() {
        let result = format_file("let x = 42;", FileIdentifier::Path(Path::new("test.rs")));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("highlight"));
    }
}
```

### Integration Tests

```rust
// Test with actual file content
#[test]
fn test_rust_highlighting() {
    let code = r#"
    fn main() {
        let x = 42;
        println!("{}", x);
    }
    "#;

    let highlighted = format_file(code, FileIdentifier::Path(Path::new("main.rs"))).unwrap();

    // Check for expected highlighting elements
    assert!(highlighted.contains("keyword"));     // fn, let
    assert!(highlighted.contains("function"));    // main, println
    assert!(highlighted.contains("number"));      // 42
}
```

## Future Enhancements

### 1. Language Server Integration

Potential integration with LSP for enhanced semantic highlighting:

```rust
// Future: LSP-based semantic highlighting
pub struct LspHighlightAdapter {
    client: LspClient,
}

impl SyntaxHighlighterAdapter for LspHighlightAdapter {
    fn write_highlighted(&self, output: &mut dyn IoWrite,
                        lang: Option<&str>, code: &str) -> std::io::Result<()> {
        // Use LSP for semantic information
    }
}
```

### 2. Real-time Highlighting

WebSocket-based real-time syntax highlighting for collaborative editing.

### 3. Plugin System

Allow users to add custom grammars without recompiling:

```rust
// Future: Plugin-based grammar loading
pub trait GrammarPlugin {
    fn name(&self) -> &str;
    fn load_grammar(&self) -> Result<Grammar, Error>;
}
```

## Security Considerations

### 1. Code Injection Prevention

All code is properly HTML-escaped before output:

```rust
// From syntax_highlight.rs:239
v_htmlescape::b_escape(line.as_bytes(), out);
```

### 2. Resource Limiting

Consider adding limits for:

- File size for highlighting
- Number of concurrent highlight requests
- Grammar loading timeout

### 3. Input Validation

Validate file paths and language tokens:

```rust
// Ensure safe file path handling
fn validate_file_path(path: &Path) -> Result<(), Error> {
    // Implement path validation logic
}
```

## Conclusion

The tree-sitter-grammar-repository system provides gnostr-gnit with enterprise-grade syntax highlighting capabilities. Its modular design, performance optimizations, and extensive language support make it an ideal solution for modern git hosting applications.

The system's flexibility allows for both static compilation for security-critical deployments and dynamic linking for development environments, while the theme system ensures consistent visual presentation across the entire application.

For developers looking to extend or modify the system, the modular architecture and comprehensive documentation should make the process straightforward and maintainable.
