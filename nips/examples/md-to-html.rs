use pulldown_cmark::{html, Parser};

fn markdown_to_html(markdown_input: &str) -> String {
    let parser = Parser::new(markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn main() {
    let markdown = r#"
# Hello, World!

This is a paragraph with **bold** and *italic* text.

* List item 1
* List item 2

[Link to Google](https://www.google.com)

```rust
fn main() {
    println!("Hello from Rust!");
}
```
"#;
    let res = markdown_to_html(markdown);
    println!("{}", res);
}
