
use crate::data_generated::EMOJI;
use std::borrow::Cow;
use regex::{
    Regex,
    Captures,
};

mod data_generated;

/// Find Unicode representation for given emoji name
///
/// The name should be without `:`, e.g. `smile`.
///
/// Case sensitive. Returns `None` if the name is not recognized.
pub fn get(name: &str) -> Option<&str> {
    EMOJI.get(name).map(|s| *s)
}

/// List all known emoji
///
/// Returns iterator of `(name, unicode)`
pub fn all() -> impl Iterator<Item=(&'static str, &'static str)> {
    EMOJI.entries.iter().map(|&x| x)
}

/// Replaces `:emoji:` in strings
///
/// ```rust
/// let r = gh_emoji::Replacer::new();
/// let unicode_text = r.replace_all("Hello :cat:!");
/// ```
pub struct Replacer {
    regex: Regex
}

impl Replacer {
    /// There is some small setup cost
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r":([a-z1238+-][a-z0-9_-]*):").unwrap(),
        }
    }

    /// Replaces all occurrences of `:emoji_names:` in the string
    ///
    /// It may return `Cow::Borrowed` if there were no emoji-like
    /// patterns in the string. Call `.to_string()` if you need
    /// `String` or `.as_ref()` to get `&str`.
    pub fn replace_all<'a>(&self, text: &'a str) -> Cow<'a, str> {
        self.regex.replace_all(text, EmojiRegexReplacer)
    }
}

struct EmojiRegexReplacer;
impl regex::Replacer for EmojiRegexReplacer {
    fn replace_append(&mut self, capts: &Captures<'_>, dst: &mut String) {
        dst.push_str(EMOJI.get(&capts[1]).copied().unwrap_or_else(|| &capts[0]));
    }
}

#[test]
fn replacer() {
    let r = Replacer::new();
    assert_eq!("hello 😄 :not_emoji_404:", r.replace_all("hello :smile: :not_emoji_404:"));
    assert_eq!(Replacer::new().replace_all(":cat: make me :smile:"), "\u{01F431} make me \u{01F604}");
}

#[test]
fn get_existing() {
    assert_eq!(get("smile"), Some("\u{01F604}"));
    assert_eq!(get("poop"),  Some("\u{01F4A9}"));
    assert_eq!(get("cat"),   Some("\u{01F431}"));
    assert_eq!(get("+1"),    Some("👍"));
    assert_eq!(get("-1"),    Some("\u{01F44E}"));
    assert_eq!(get("8ball"), Some("\u{01F3B1}"));
}

#[test]
fn get_nonexistent() {
    assert_eq!(get("stuff"), None);
    assert_eq!(get("++"), None);
    assert_eq!(get("--"), None);
    assert_eq!(get("666"), None);
}
