# Tree-Sitter Grammar Repository in gnostr-gnit

This document explains how the `tree-sitter-grammar-repository` system provides syntax highlighting for the gnostr-gnit git hosting application.

## Overview

The gnostr-gnit project uses tree-sitter to provide professional-grade syntax highlighting for code files, diffs, and README content. The system is built around a custom `tree-sitter-grammar-repository` crate that manages 200+ language grammars.

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    gnostr-gnit Web Server                    │
├─────────────────────────────────────────────────────────────────┤
│  Request Router                                              │
│  ├── /repo/:name/tree/*     → FileView                   │
│  ├── /repo/:name/commit/*   → CommitView (diffs)          │
│  └── /repo/:name/summary    → RepositoryView (README)     │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                Syntax Highlighting Engine                      │
├─────────────────────────────────────────────────────────────────┤
│  syntax_highlight.rs                                          │
│  ├── format_file()           - File content highlighting      │
│  ├── ComrakHighlightAdapter   - Markdown code fence support   │
│  └── prime_highlighters()     - Initialize grammars          │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│              tree-sitter-grammar-repository                     │
├─────────────────────────────────────────────────────────────────┤
│  Grammar Registry                                            │
│  ├── 200+ Language Grammars                                │
│  ├── Highlighter Configurations                             │
│  ├── Lazy Loading System                                   │
│  └── CSS Theme Generation                                  │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                Build System & Grammars                         │
├─────────────────────────────────────────────────────────────────┤
│  ├── Static Compilation (default)                             │
│  ├── Dynamic Loading (TREE_SITTER_GRAMMAR_LIB_DIR)          │
│  ├── Helix Editor Grammar Sources                           │
│  └── Threadpool Parallel Build                              │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Grammar Repository Crate

**Location**: `tree-sitter-grammar-repository/`

The grammar repository is a workspace dependency that:

- **Fetches grammars** from Helix editor's repository during build
- **Compiles 200+ tree-sitter parsers** for different programming languages
- **Generates language registry** as Rust code at build time
- **Supports both static and dynamic linking** modes

**Key Features**:

```rust
// Generated at build time - includes all supported languages
pub enum Grammar {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    // ... 200+ total languages
}

// Each grammar provides configuration data
pub struct GrammarConfig {
    pub name: &'static str,
    pub highlights_query: &'static str,
    pub injection_query: Option<&'static str>,
    pub locals_query: Option<&'static str>,
    pub language: unsafe extern "C" fn() -> *mut TSLanguage,
}
```

### 2. Syntax Highlighting Engine

**Location**: `src/syntax_highlight.rs`

#### Core Functions

**`format_file()`** - Main entry point for file highlighting:

```rust
pub fn format_file(content: &str, identifier: FileIdentifier<'_>) -> anyhow::Result<String> {
    let Some(config) = fetch_highlighter_config(&identifier.path) else {
        return Ok("<pre>".to_string() + html_escape::encode_text_minimal(content) + "</pre>");
    };

    let mut highlighter = Highlighter::new();
    let mut renderer = HtmlRenderer::new(&HIGHLIGHT_NAMES);

    // Apply highlighting theme
    highlighter.configure(config);
    renderer.configure(&HIGHLIGHT_NAMES);

    // Generate highlighted HTML
    highlighter.highlight(&mut renderer, content.as_bytes(), None, |_| None)?;

    Ok(renderer.finalize())
}
```

**`ComrakHighlightAdapter`** - Markdown code fence support:

```rust
impl comrak::plugins::syntect::SyntaxHighlighterAdapter for ComrakHighlightAdapter {
    fn write_highlighted(&self, output: &mut dyn Write, lang: Option<&str>, code: &str) -> io::Result<()> {
        let highlighted = if let Some(lang) = lang {
            format_file(code, FileIdentifier::new("", Path::new(lang), false))
                .unwrap_or_else(|_| plain_text(code))
        } else {
            plain_text(code)
        };

        write!(output, "{}", highlighted)
    }
}
```

### 3. Integration Points

#### File Views

**Location**: `src/methods/repo/tree.rs`

```rust
#[derive(Template)]
#[template(path = "repo/file.html")]
pub struct FileView {
    pub repo: Repository,
    pub repo_path: PathBuf,
    pub file: FileWithContent,  // Contains highlighted content
    pub branch: Option<Arc<str>>,
}
```

#### Diff Views

**Location**: `src/git.rs`

```rust
impl Commit {
    pub async fn diff_highlighted(self: Arc<Self>) -> anyhow::Result<(String, String)> {
        // Generate diff stats with syntax highlighting
        let diff_stats = format_file(&self.diff_stats, FileIdentifier::new("", Path::new("diff"), false))?;
        let diff_content = format_file(&self.diff, FileIdentifier::new("", Path::new("diff"), false))?;

        Ok((diff_stats, diff_content))
    }
}
```

#### README Rendering

**Location**: `src/git.rs`

```rust
impl OpenRepository {
    pub async fn readme(self: Arc<Self>) -> anyhow::Result<Option<(ReadmeFormat, Arc<str>)>> {
        // Find README files in repository
        for readme_file in README_FILES {
            if let Some(content) = self.file_content(readme_file).await? {
                let format = ReadmeFormat::from_filename(readme_file);
                return Ok(Some((format, content)));
            }
        }
        Ok(None)
    }
}
```

## Configuration Options

### Environment Variables

**`TREE_SITTER_GRAMMAR_LIB_DIR`**

- **Purpose**: Enable dynamic loading of pre-compiled grammars
- **Usage**: Set to directory containing `.so`/`.dll`/`.dylib` grammar files
- **Benefit**: Faster build times, reduced binary size
- **Example**:
  ```bash
  export TREE_SITTER_GRAMMAR_LIB_DIR=/usr/local/lib/tree-sitter
  ```

**Build Modes**:

1. **Static (Default)**:

   ```bash
   cargo build
   # Compiles all grammars during build
   # Larger binary, self-contained
   ```

2. **Dynamic**:
   ```bash
   export TREE_SITTER_GRAMMAR_LIB_DIR=/path/to/grammars
   cargo build
   # Uses pre-compiled libraries
   # Smaller binary, requires runtime libraries
   ```

### Supported Languages

The system supports **200+ languages** including:

| Category          | Languages                                                          |
| ----------------- | ------------------------------------------------------------------ |
| **Programming**   | Rust, Go, Python, JavaScript, TypeScript, Java, C++, C#, PHP, Ruby |
| **Web**           | HTML, CSS, Svelte, Vue, React (JSX), Angular, GraphQL              |
| **Configuration** | Dockerfile, TOML, YAML, JSON, INI, XML                             |
| **Build Systems** | Makefile, CMake, Bazel, Gradle, Maven                              |
| **Databases**     | SQL, PL/SQL, T-SQL, GraphQL                                        |
| **DevOps**        | Terraform, Ansible, Kubernetes YAML                                |
| **Specialized**   | Solidity, LLVM IR, LaTeX, Markdown, Protocol Buffers               |

## Build Process

### Grammar Compilation Flow

```
Build Start
     │
     ▼
┌─────────────────────┐
│ Check Environment   │
│ TREE_SITTER_...?   │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐    ┌─────────────────────┐
│ Static Mode        │    │ Dynamic Mode       │
│                   │    │                   │
│ 1. Download      │    │ 1. Skip download  │
│    Helix grammars │    │ 2. Skip compile   │
│                   │    │ 3. Load .so files │
│ 2. Compile        │    │    at runtime     │
│    each grammar   │    │                   │
│                   │    │                   │
│ 3. Generate       │    │ 4. Generate       │
│    Rust registry   │    │    Rust registry   │
└─────────┬───────────┘    └─────────┬───────────┘
          │                      │
          └──────────┬───────────┘
                     ▼
          ┌─────────────────────┐
          │ Link with Main     │
          │ Application        │
          └─────────┬───────────┘
                    │
                    ▼
          ┌─────────────────────┐
          │ Ready for Use      │
          └───────────────────┘
```

### Build Steps

1. **Grammar Download**:

   ```bash
   # tree-sitter-grammar-repository/build.rs
   let helix_repo = "https://github.com/helix-editor/helix";
   let grammar_dir = "helix/runtime/grammars/";
   ```

2. **Parallel Compilation**:

   ```rust
   // Threadpool compilation for performance
   let threadpool = ThreadPool::new(num_cpus::get());
   for grammar in grammars {
       threadpool.execute(|| {
           // Compile grammar to static library
           compile_grammar(grammar);
       });
   }
   ```

3. **Registry Generation**:
   ```rust
   // Generate Rust code at build time
   generate_grammar_registry(&compiled_grammars);
   // Outputs: src/grammar_registry.rs
   ```

## Performance Considerations

### Lazy Initialization

Grammars are loaded on-demand:

```rust
static HIGHLIGHTER_CONFIGS: LazyLock<Vec<HighlightConfiguration>> = LazyLock::new(|| {
    Grammar::VARIANTS
        .iter()
        .copied()
        .map(Grammar::highlight_configuration_params)
        .map(|v| {
            let mut configuration = HighlightConfiguration::new(
                v.language.into(),
                v.name,
                v.highlights_query,
                v.injection_query,
                v.locals_query,
            ).unwrap_or_else(|e| panic!("bad query for {}: {e}", v.name));

            configuration.configure(&HIGHLIGHT_NAMES);
            configuration
        })
        .collect()
});
```

### Caching Strategy

- **Build Cache**: Compiled grammars cached in `target/debug/build/`
- **Runtime Cache**: Syntax highlighting results cached where possible
- **Configuration Cache**: Highlighter configurations cached after first use

### Memory Optimization

```rust
// Use reference counting for shared grammar configurations
pub fn fetch_highlighter_config(file: &Path) -> Option<&'static HighlightConfiguration> {
    // Direct access to pre-configured highlighters
    // No runtime allocation for known file types
}
```

## Theme System

### CSS Generation

**Location**: `src/theme.rs`

```rust
pub fn build_css(theme: &Theme) -> String {
    let mut css = String::new();

    for (style_class, style) in &theme.styles {
        css.push_str(&format!(
            ".{} {{ color: {}; font-weight: {}; font-style: {}; }}\n",
            style_class,
            style.color.unwrap_or_default(),
            style.font_weight.unwrap_or("normal"),
            style.font_style.unwrap_or("normal")
        ));
    }

    css
}
```

### Theme Configuration

Themes defined in `themes/` directory:

```toml
# themes/solarized_dark.toml
[styles]
"attribute" = { color = "#b58900" }
"comment" = { color = "#586e75", font_style = "italic" }
"constant" = { color = "#cb4b16" }
"function" = { color = "#268bd2" }
"keyword" = { color = "#859900", font_weight = "bold" }
# ... more style definitions
```

### CSS Hash Management

```rust
// In main binary (src/bin/gnostr-gnit.rs)
let css_hash = build_asset_hash(theme_css);
HIGHLIGHT_CSS_HASH.set(css_hash).unwrap();

// Template usage
<link rel="stylesheet" href="/highlight-{{ css_hash }}.css" />
```

## Adding New Languages

### Method 1: Static Integration (Recommended)

1. **Add Grammar to Helix Repository**:

   ```bash
   # Fork helix-editor/helix
   # Add grammar to runtime/grammars/
   # Submit PR
   ```

2. **Update Grammar Repository**:

   ```bash
   # tree-sitter-grammar-repository/Cargo.toml
   # Update to latest Helix commit
   ```

3. **Rebuild**:
   ```bash
   cargo build
   # Grammar automatically included
   ```

### Method 2: Custom Grammar

1. **Add Grammar Source**:

   ```bash
   mkdir -p custom-grammars/mylang
   # Add grammar files: grammar.js, queries/highlights.scm, etc.
   ```

2. **Update Build Script**:

   ```rust
   // tree-sitter-grammar-repository/build.rs
   fn add_custom_grammar(name: &str, path: &Path) {
       // Compile and include custom grammar
   }
   ```

3. **Generate Registry Entry**:
   ```rust
   // Add to generated grammar registry
   Grammar::MyLang => GrammarConfig {
       name: "mylang",
       highlights_query: include_str!("../custom-grammars/mylang/queries/highlights.scm"),
       // ... other fields
   }
   ```

## Usage Examples

### Basic File Highlighting

```rust
use gnostr_gnit::syntax_highlight::format_file;
use std::path::Path;

fn highlight_rust_code(content: &str) -> anyhow::Result<String> {
    format_file(content, gnostr_gnit::syntax_highlight::FileIdentifier::new(
        "main.rs",
        Path::new("main.rs"),
        false
    ))
}

let highlighted = highlight_rust_code(r#"
fn main() {
    println!("Hello, world!");
}
"#)?;
```

### Markdown with Code Blocks

```rust
use comrak::{markdown_to_html, Options};
use gnostr_gnit::syntax_highlight::ComrakHighlightAdapter;

fn render_markdown(content: &str) -> String {
    let mut options = Options::default();
    options.extension.syntax_highlighting = Some(Box::new(ComrakHighlightAdapter));

    markdown_to_html(content, &options)
}
```

### Custom Theme

```rust
use gnostr_gnit::theme::{Theme, build_css};

fn create_custom_theme() -> String {
    let theme = Theme {
        styles: vec![
            ("keyword".to_string(), Style {
                color: Some("#ff6b6b".to_string()),
                font_weight: Some("bold".to_string()),
                font_style: None,
            }),
            ("string".to_string(), Style {
                color: Some("#51cf66".to_string()),
                font_weight: None,
                font_style: None,
            }),
            // ... more styles
        ],
    };

    build_css(&theme)
}
```

## Troubleshooting

### Common Issues

#### 1. Grammar Compilation Warnings

```
warning: tree-sitter-grammar-repository: archive library: ... the table of contents is empty
```

**Cause**: Grammar compiled but no exported symbols
**Solution**: Harmless warning, can be ignored

#### 2. Missing Syntax Highlighting

**Symptoms**: Code displays as plain text
**Diagnosis**:

```rust
// Check if grammar exists for file type
let config = fetch_highlighter_config(&file_path);
if config.is_none() {
    eprintln!("No grammar for {:?}", file_path);
}
```

**Solutions**:

- Check supported languages list
- Verify file extension mapping
- Ensure grammar was compiled successfully

#### 3. Build Performance Issues

**Slow Builds**:

```bash
# Enable dynamic loading for faster builds
export TREE_SITTER_GRAMMAR_LIB_DIR=/path/to/prebuilt/grammars
cargo build
```

**High Memory Usage**:

- Use `cargo build --release` for optimized build
- Consider reducing grammar set in configuration

#### 4. Runtime Errors

**Panics in Templates**:

```html
<!-- Template error with uninitialized CSS hashes -->
<!-- Fix: Use conditional checks -->
{% if let Some(css_hash) = crate::HIGHLIGHT_CSS_HASH.get() %}
<link rel="stylesheet" href="/highlight-{{ css_hash }}.css" />
{% endif %}
```

### Debug Commands

```bash
# List all supported languages
cargo tree | grep tree-sitter

# Check grammar compilation
ls -la target/debug/build/tree-sitter-grammar-repository-*/out/

# Verify dynamic libraries
ls $TREE_SITTER_GRAMMAR_LIB_DIR/*.so

# Test highlighting manually
echo 'fn main() {}' | cargo run --bin test-highlighter
```

## Testing & Validation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rust_highlighting() {
        let code = r#"fn main() { println!("test"); }"#;
        let result = format_file(code, FileIdentifier::new("", Path::new("test.rs"), false));
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<span"));
    }

    #[test]
    fn test_unsupported_language() {
        let code = "some content";
        let result = format_file(code, FileIdentifier::new("", Path::new("test.unknown"), false));
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<pre>"));
        assert!(!html.contains("<span"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_file_view_highlighting() {
    // Start test server
    // Request file with syntax highlighting
    // Verify HTML contains expected CSS classes
}
```

### Performance Testing

```rust
use std::time::Instant;

fn benchmark_highlighting() {
    let large_code = include_str!("large_file.rs");
    let start = Instant::now();

    for _ in 0..100 {
        let _ = format_file(large_code, FileIdentifier::new("", Path::new("test.rs"), false));
    }

    let duration = start.elapsed();
    println!("100 highlights took {:?}", duration);
}
```

## Future Enhancements

### Potential Improvements

1. **Language Server Protocol (LSP) Integration**:
   - Provide code completion
   - Error diagnostics
   - Go-to-definition

2. **Incremental Highlighting**:
   - Re-highlight only changed lines
   - Better performance for large files

3. **WebAssembly Compilation**:
   - Client-side syntax highlighting
   - Reduced server load

4. **Plugin System**:
   - Custom grammar loading
   - User-defined themes

5. **Semantic Highlighting**:
   - Language-aware token classification
   - Better accuracy than syntactic highlighting

## Security Considerations

### Input Validation

```rust
// Sanitize file paths to prevent directory traversal
fn sanitize_path(path: &Path) -> PathBuf {
    let path = path.components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect::<PathBuf>();
    path
}
```

### Resource Limits

```rust
// Limit file size for highlighting
const MAX_HIGHLIGHT_SIZE: usize = 1024 * 1024; // 1MB

pub fn safe_format_file(content: &str, identifier: FileIdentifier<'_>) -> anyhow::Result<String> {
    if content.len() > MAX_HIGHLIGHT_SIZE {
        return Ok("<pre>File too large for syntax highlighting</pre>".to_string());
    }

    format_file(content, identifier)
}
```

### Content Security Policy

```html
<!-- Generated HTML uses safe classes only -->
<span class="keyword">fn</span>
<span class="function">main</span>
<span class="punctuation">{</span>
<!-- No inline styles or scripts -->
```

---

This documentation provides a comprehensive understanding of how tree-sitter-grammar-repository enables professional syntax highlighting in the gnostr-gnit git hosting application. The system is designed for performance, extensibility, and maintainability while supporting a wide range of programming languages.
