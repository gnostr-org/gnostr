use tree_sitter_grammar_repository::{Grammar, HighlightConfigurationParams};
use std::{cell::RefCell, collections::HashMap, ops::Range, path::Path};
use tree_sitter::Language;
use tree_sitter::ffi;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTag {
    Attribute,
    Comment,
    Constant,
    ConstantBuiltin,
    Constructor,
    Embedded,
    Function,
    FunctionBuiltin,
    Keyword,
    Module,
    Number,
    Operator,
    Property,
    PunctuationBracket,
    PunctuationDelimiter,
    String,
    StringSpecial,
    Tag,
    TypeBuiltin,
    TypeRegular,
    VariableBuiltin,
    VariableParameter,
}

impl AsRef<str> for SyntaxTag {
    fn as_ref(&self) -> &str {
        match self {
            SyntaxTag::Attribute => "attribute",
            SyntaxTag::Comment => "comment",
            SyntaxTag::ConstantBuiltin => "constant.builtin",
            SyntaxTag::Constant => "constant",
            SyntaxTag::Constructor => "constructor",
            SyntaxTag::Embedded => "embedded",
            SyntaxTag::FunctionBuiltin => "function.builtin",
            SyntaxTag::Function => "function",
            SyntaxTag::Keyword => "keyword",
            SyntaxTag::Number => "number",
            SyntaxTag::Module => "module",
            SyntaxTag::Property => "property",
            SyntaxTag::Operator => "operator",
            SyntaxTag::PunctuationBracket => "punctuation.bracket",
            SyntaxTag::PunctuationDelimiter => "punctuation.delimiter",
            SyntaxTag::StringSpecial => "string.special",
            SyntaxTag::String => "string",
            SyntaxTag::Tag => "tag",
            SyntaxTag::TypeRegular => "type",
            SyntaxTag::TypeBuiltin => "type.builtin",
            SyntaxTag::VariableBuiltin => "variable.builtin",
            SyntaxTag::VariableParameter => "variable.parameter",
        }
    }
}

/// Safety: The `ptr` must be a valid `*const ffi::TSLanguage`.
unsafe fn unsafe_language_from_ptr(ptr: *const ffi::TSLanguage) -> Language {
    std::mem::transmute(ptr)
}

fn tags_by_highlight_index() -> [SyntaxTag; 22] {
    [
        SyntaxTag::Attribute,
        SyntaxTag::Comment,
        SyntaxTag::ConstantBuiltin,
        SyntaxTag::Constant,
        SyntaxTag::Constructor,
        SyntaxTag::Embedded,
        SyntaxTag::FunctionBuiltin,
        SyntaxTag::Function,
        SyntaxTag::Keyword,
        SyntaxTag::Number,
        SyntaxTag::Module,
        SyntaxTag::Property,
        SyntaxTag::Operator,
        SyntaxTag::PunctuationBracket,
        SyntaxTag::PunctuationDelimiter,
        SyntaxTag::StringSpecial,
        SyntaxTag::String,
        SyntaxTag::Tag,
        SyntaxTag::TypeRegular,
        SyntaxTag::TypeBuiltin,
        SyntaxTag::VariableBuiltin,
        SyntaxTag::VariableParameter,
    ]
}

fn determine_lang(path: &Path) -> Option<(Grammar, Language)> {

    let file_language = tree_sitter_grammar_repository::Language::from_file_name(path)?;

    let grammar_variant = file_language.grammar();

    let params = Grammar::highlight_configuration_params(grammar_variant);

    Some((grammar_variant, unsafe { tree_sitter::Language((params.language.into_raw())() as *const tree_sitter::ffi::TSLanguage) }))

}

fn create_highlight_config(grammar_variant: Grammar, language: &Language) -> HighlightConfiguration {
    let params = Grammar::highlight_configuration_params(grammar_variant);

    let mut highlight_config =
        HighlightConfiguration::new(*language, params.highlights_query, params.injection_query, params.locals_query)
            .unwrap();

    highlight_config.configure(&tags_by_highlight_index());
    highlight_config
}

thread_local! {
    pub static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
    pub static LANG_CONFIGS: RefCell<HashMap<Grammar, HighlightConfiguration>> = RefCell::new(HashMap::new());
}

pub fn parse<'a>(path: &'a Path, content: &'a str) -> Vec<(Range<usize>, SyntaxTag)> {
    let tags = tags_by_highlight_index();

    let Some((grammar_variant, language)) = determine_lang(path) else {
        return vec![];
    };

    LANG_CONFIGS.with(|highlight_configs| {
        let mut highlight_configs_borrow = highlight_configs.borrow_mut();
        let config = highlight_configs_borrow
            .entry(grammar_variant)
            .or_insert_with_key(|g_variant| create_highlight_config(*g_variant, &language));

        HIGHLIGHTER.with_borrow_mut(|highlighter| {
            highlighter
                .highlight(config, content.as_bytes(), None, |_| None)
                .unwrap()
                .scan(None, move |current_tag, event| match event.unwrap() {
                    HighlightEvent::Source { start, end } => Some(Some((start..end, *current_tag))),
                    HighlightEvent::HighlightStart(Highlight(highlight)) => {
                        *current_tag = Some(tags[highlight]);
                        Some(None)
                    }
                    HighlightEvent::HighlightEnd => {
                        *current_tag = None;
                        Some(None)
                    }
                })
                .flatten()
                .filter_map(|(range, maybe_tag)| maybe_tag.map(|tag| (range, tag)))
                .collect()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight() {
        let path = Path::new("test.rs");
        let content = r#"
fn main() {
    println!("Hello, world!");
}
"#;

        let syntax = parse(path, content);
        let syntax_with_content = syntax
            .into_iter()
            .map(|(range, style)| {
                (
                    std::str::from_utf8(&content.as_bytes()[range.clone()]).unwrap(),
                    style,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            syntax_with_content,
            vec![
                ("fn", SyntaxTag::Keyword),
                ("main", SyntaxTag::Function),
                ("(", SyntaxTag::PunctuationBracket),
                (")", SyntaxTag::PunctuationBracket),
                ("{", SyntaxTag::PunctuationBracket),
                ("println", SyntaxTag::Function),
                ("!", SyntaxTag::Function),
                ("(", SyntaxTag::PunctuationBracket),
                ("\"Hello, world!\"", SyntaxTag::String),
                (")", SyntaxTag::PunctuationBracket),
                (";", SyntaxTag::PunctuationDelimiter),
                ("}", SyntaxTag::PunctuationBracket),
            ]
        );
    }
}
