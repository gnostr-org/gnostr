use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    fmt::Write,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use anyhow::{bail, Context};
use chrono::TimeZone;
use heck::{ToSnakeCase, ToUpperCamelCase};
use quote::{format_ident, quote};
use serde::Deserialize;
use threadpool::ThreadPool;

const GRAMMAR_REPOSITORY_URL: &str = "https://github.com/helix-editor/helix";
const GRAMMAR_REPOSITORY_REF: &str = "82dd96369302f60a9c83a2d54d021458f82bcd36";
const GRAMMAR_REPOSITORY_CONFIG_PATH: &str = "languages.toml";
const GRAMMAR_CACHE_NAMESPACE: &str = "gnostr/grammar";

static BLACKLISTED_MODULES: &[&str] = &[
    // these languages all don't have corresponding grammars
    "cabal",
    "idris",
    "llvm-mir-yaml",
    "prolog",
    "mint",
    "hare",
    "wren",
    // doesn't compile on macos
    "gemini",
    "gotmpl",
    "helm",
    "rust",
    "sway",
    "toml",
    "awk",
    "protobuf",
    "elixir",
    "fish",
    "mojo",
    "janet_simple",
    "c",
    "cpp",
    "c_sharp",
    "cel",
    "spicedb",
    "go",
    "gomod",
    "gowork",
    "typespec",
    "tsx",
    "css",
    "scss",
    "nickel",
    "nix",
    "php",
    "php_only",
    "blade",
    "twig",
    "latex",
    "bibtex",
    "lean",
    "lpf",
    "julia",
    "java",
    "smali",
    "ledger",
    "beancount",
    "ocaml",
    "ocaml_interface",
    "lua",
    "svelte",
    "vue",
    "haskell",
    "haskell_persistent",
    "purescript",
    "zig",
    "tsq",
    "cmake",
    "make",
    "glsl",
    "perl",
    "pod",
    "comment",
    "wgsl",
    "llvm",
    "llvm_mir",
    "tablegen",
    "dart",
    "scala",
    "dockerfile",
    "regex",
    "graphql",
    "elm",
    "iex",
    "rescript",
    "erlang",
    "kotlin",
    "org",
    "solidity",
    "gleam",
    "ron",
    "robot",
    "r",
    "swift",
    "embedded_template",
    "eex",
    "heex",
    "sql",
    "gdscript",
    "godot_resource",
    "nu",
    "vala",
    "devicetree",
    "cairo",
    "cpon",
    "odin",
    "meson",
    "sshclientconfig",
    "v",
    "verilog",
    "edoc",
    "jsdoc",
    "openscad",
    "prisma",
    "clojure",
    "elvish",
    "fortran",
    "ungrammar",
    "dot",
    "cue",
    "slint",
    "task",
    "xit",
    "esdl",
    "pascal",
    "sml",
    "jsonnet",
    "ada",
    "astro",
    "bass",
    "wat",
    "wast",
    "d",
    "vhs",
    "kdl",
    "dtd",
    "wit",
    "ini",
    "inko",
    "bicep",
    "mermaid",
    "matlab",
    "ponylang",
    "dhall",
    "pem",
    "passwd",
    "hosts",
    "uxntal",
    "yuck",
    "prql",
    "po",
    "nasm",
    "gas",
    "rst",
    "capnp",
    "smithy",
    "vhdl",
    "rego",
    "nim",
    "hurl",
    "markdoc",
    "opencl",
    "just",
    "gn",
    "blueprint",
    "forth",
    "fsharp",
    "t32",
    "typst",
    /* "jinja2", */
    "jjdescription",
    "jq",
    "unison",
    "todotxt",
    "strace",
    "agda",
    "templ",
    "dbml",
    "bitbake",
    "log",
    "hoon",
    "hocon",
    "tfvars",
    "koka",
    "tact",
    "pkl",
    "groovy",
    "fidl",
    "powershell",
    "ld",
    "hyprlang",
    "tcl",
    "supercollider",
    "glimmer",
    "ohm",
    "earthfile",
    "adl",
    "ldif",
    "xtc",
    "r#move",
    "pest",
    "elisp",
    "gherkin",
    "thrift",
    "circom",
];

/// Grammars that fail to compile on macOS (beyond the always-blacklisted set above).
static MACOS_BLACKLISTED: &[&str] = &[];

/// Grammars that fail to compile on musl-libc targets (statically linked Linux).
static MUSL_BLACKLISTED: &[&str] = &[];

/// Grammars that fail to compile on Windows targets.
static WINDOWS_BLACKLISTED: &[&str] = &[];

/// Returns true if the grammar should be skipped for the current build target.
fn is_blacklisted(name: &str) -> bool {
    if BLACKLISTED_MODULES.contains(&name) {
        return true;
    }
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    match target_os.as_str() {
        "macos" if MACOS_BLACKLISTED.contains(&name) => true,
        "windows" if WINDOWS_BLACKLISTED.contains(&name) => true,
        _ if target_env == "musl" && MUSL_BLACKLISTED.contains(&name) => true,
        _ => false,
    }
}

fn main() -> anyhow::Result<()> {
    report_build_name();
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => {
            // Tree-sitter grammars compiled from C++ sources need the C++ runtime on Darwin.
            println!("cargo::rustc-link-lib=c++");
        }
        Ok("linux") => {
            // Linux CI links the generated scanner sources against libstdc++.
            println!("cargo::rustc-link-lib=stdc++");
        }
        _ => {}
    }
    //println!("cargo:warning=DEBUG: gnostr-grammar build.rs is executing.");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").context("OUT_DIR not set by rustc")?);
    println!("out_dir={}", &out_dir.display());
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=GRAMMAR_REPOSITORY_REF");

    let root = std::env::var("TREE_SITTER_GRAMMAR_LIB_DIR").ok();
    println!("root={:?}", &root);

    println!("cargo::rerun-if-env-changed=TREE_SITTER_GRAMMAR_LIB_DIR");

    let (root, dylib) = if let Some(root) = root.as_deref() {
        (Path::new(root), true)
    } else {
        (out_dir.as_path(), false)
    };

    let (config, query_path) = if dylib {
        let config: HelixLanguages = toml::from_str(
            &fs::read_to_string(root.join("languages.toml"))
                .context("failed to read languages.toml")?,
        )
        .context("failed to parse helix languages.toml")?;

        println!("cargo::rustc-link-search=native={}", root.display());

        for grammar in &config.grammar {
            if is_blacklisted(&grammar.name) {
                continue;
            }

            println!("cargo::rustc-link-lib=dylib=tree-sitter-{}", grammar.name);
        }

        (config, root.join("queries"))
    } else {
        let cache_root = grammar_cache_root()?;
        fs::create_dir_all(&cache_root)?;

        let grammar_ref = grammar_repository_ref();
        let helix_root = cache_root.join("helix").join(&grammar_ref);
        if let Some(parent) = helix_root.parent() {
            fs::create_dir_all(parent)?;
        }

        ensure_helix_repository(&helix_root)?;

        let config: HelixLanguages = toml::from_str(
            &fs::read_to_string(helix_root.join(GRAMMAR_REPOSITORY_CONFIG_PATH))
                .context("failed to read helix languages.toml")?,
        )
        .context("failed to parse helix languages.toml")?;

        let source_root = cache_root.join("sources").join(&grammar_ref);
        let build_root = cache_root.join("build").join(target_triple()).join(&grammar_ref);
        fs::create_dir_all(&source_root)?;
        fs::create_dir_all(&build_root)?;
        fetch_and_build_grammar(config.grammar.clone(), &source_root, &build_root)?;

        (config, helix_root.join("runtime/queries"))
    };

    let mut grammar_defs = Vec::new();
    for grammar in &config.grammar {
        let name = &grammar.name;
        if let Some(tokens) =
            build_language_module(name, query_path.as_path()).with_context(|| name.to_string())?
        {
            grammar_defs.push(tokens);
        }
    }
    fs::write(
        out_dir.join("grammar.defs.rs"),
        prettyplease::unparse(
            &syn::parse2(quote!(#(#grammar_defs)*)).context("failed to parse grammar defs")?,
        ),
    )
    .context("failed to write grammar defs")?;

    let registry = build_grammar_registry(config.grammar.iter().map(|v| v.name.clone()));
    fs::write(
        out_dir.join("grammar.registry.rs"),
        prettyplease::unparse(&syn::parse2(registry).context("failed to parse grammar registry")?),
    )
    .context("failed to write grammar registry")?;

    let language = build_language_registry(config.language)?;
    fs::write(
        out_dir.join("language.registry.rs"),
        prettyplease::unparse(&syn::parse2(language)?),
    )?;

    Ok(())
}

fn report_build_name() {
    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();
    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!("{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "{}@{} {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };

    println!("cargo:warning=buildname '{build_name}'");
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
}

fn ensure_helix_repository(destination: &Path) -> anyhow::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    if destination.join(GRAMMAR_REPOSITORY_CONFIG_PATH).exists() {
        return Ok(());
    }

    match fetch_git_repository(GRAMMAR_REPOSITORY_URL, GRAMMAR_REPOSITORY_REF, destination) {
        Ok(()) => Ok(()),
        Err(err) => {
            println!(
                "cargo:warning=git fetch for helix failed ({err}); falling back to archive download"
            );
            if destination.exists() {
                fs::remove_dir_all(destination)?;
            }
            fetch_helix_archive(destination).context(GRAMMAR_REPOSITORY_URL)
        }
    }
}

fn grammar_repository_ref() -> String {
    std::env::var("GRAMMAR_REPOSITORY_REF").unwrap_or_else(|_| GRAMMAR_REPOSITORY_REF.to_string())
}

fn grammar_cache_root() -> anyhow::Result<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from) {
        candidates.push(cache_home.join(GRAMMAR_CACHE_NAMESPACE));
    }

    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        candidates.push(home.join(".cache").join(GRAMMAR_CACHE_NAMESPACE));
    }

    if let Ok(out_dir) = std::env::var("OUT_DIR").map(PathBuf::from) {
        candidates.push(out_dir.join("grammar-cache"));
    }

    candidates.push(std::env::temp_dir().join(GRAMMAR_CACHE_NAMESPACE));

    for candidate in candidates {
        if fs::create_dir_all(&candidate).is_ok() {
            return Ok(candidate);
        }
    }

    bail!("unable to determine writable grammar cache directory")
}

fn target_triple() -> String {
    std::env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_string())
}

fn get_git_hash() -> String {
    if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
        return commit[..7].to_string();
    }

    let commit = Command::new("git")
        .arg("rev-parse")
        .arg("--short=7")
        .arg("--verify")
        .arg("HEAD")
        .output();

    if let Ok(commit_output) = commit {
        let commit_string = String::from_utf8_lossy(&commit_output.stdout);
        return commit_string.lines().next().unwrap_or("").into();
    }

    panic!("Can not get git commit: {}", commit.unwrap_err());
}

fn build_language_registry(
    language_definition: Vec<LanguageDefinition>,
) -> anyhow::Result<proc_macro2::TokenStream> {
    let mut camel = Vec::new();
    let mut grammars = Vec::new();

    let mut globs = Vec::new();
    let mut globs_to_camel = Vec::new();

    let mut injection_regex = Vec::new();
    let mut injection_regex_str_len = Vec::new();
    let mut regex_to_camel = Vec::new();

    for language in &language_definition {
        if is_blacklisted(&language.name) {
            continue;
        }

        let camel_cased_name = format_ident!("{}", language.name.to_upper_camel_case());
        camel.push(camel_cased_name.clone());

        let grammar = language
            .grammar
            .as_deref()
            .unwrap_or(language.name.as_str());
        grammars.push(format_ident!("{}", grammar.to_upper_camel_case()));

        for ty in &language.file_types {
            match ty {
                FileType::Glob { glob } => globs.push(Cow::Borrowed(glob)),
                FileType::Extension(ext) => globs.push(Cow::Owned(format!("*.{ext}"))),
            }

            globs_to_camel.push(camel_cased_name.clone());
        }

        if let Some(regex) = language.injection_regex.as_deref() {
            injection_regex.push(format!("^{regex}$"));
            injection_regex_str_len.push(regex.len());
            regex_to_camel.push(camel_cased_name.clone());
        }
    }

    let injection_regex_len = injection_regex.len();
    let globs_array_len = globs.len();
    let globs_string_len = globs.iter().map(|v| v.len()).collect::<Vec<_>>();

    Ok(quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Language {
            #(#camel),*
        }

        impl Language {
            pub const VARIANTS: &[Self] = &[
                #(Self::#camel),*
            ];

            pub const fn grammar(self) -> Grammar {
                match self {
                    #(Self::#camel => Grammar::#grammars),*
                }
            }

            pub fn from_file_name<P: AsRef<::std::path::Path>>(name: P) -> Option<Self> {
                const LENGTHS: [usize; #globs_array_len] = [#(#globs_string_len),*];
                const GLOB_TO_VARIANT: [Language; #globs_array_len] = [#(Language::#globs_to_camel),*];

                thread_local! {
                    static GLOB: ::std::cell::LazyCell<::globset::GlobSet> = ::std::cell::LazyCell::new(|| {
                        ::globset::GlobSetBuilder::new()
                            #(.add(::globset::Glob::new(#globs).unwrap()))*
                            .build()
                            .unwrap()
                    });
                }

                let mut max = usize::MAX;
                let mut curr = None;

                GLOB.with(|glob| {
                    for m in glob.matches(name) {
                        let curr_length = LENGTHS[m];

                        if curr_length < max {
                            max = curr_length;
                            curr = Some(GLOB_TO_VARIANT[m]);
                        }
                    }
                });

                curr
            }

            pub fn from_injection(name: &str) -> Option<Self> {
                const LENGTHS: [usize; #injection_regex_len] = [#(#injection_regex_str_len),*];
                const REGEX_TO_VARIANT: [Language; #injection_regex_len] = [#(Language::#regex_to_camel),*];

                thread_local! {
                    static REGEX: ::std::cell::LazyCell<::regex::RegexSet> = ::std::cell::LazyCell::new(|| {
                        ::regex::RegexSet::new([
                            #(#injection_regex),*
                        ])
                        .unwrap()
                    });
                }

                let mut max = usize::MAX;
                let mut curr = None;

                REGEX.with(|regex| {
                    for m in regex.matches(name) {
                        let curr_length = LENGTHS[m];

                        if curr_length < max {
                            max = curr_length;
                            curr = Some(REGEX_TO_VARIANT[m]);
                        }
                    }
                });

                curr
            }
        }
    })
}

fn build_grammar_registry(names: impl Iterator<Item = String>) -> proc_macro2::TokenStream {
    let (ids, plain, camel, snake) = names
        .filter(|name| !is_blacklisted(name))
        .enumerate()
        .fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut ids, mut plain_acc, mut camel_acc, mut snake_acc), (i, name)| {
                camel_acc.push(format_ident!("{}", name.to_upper_camel_case()));

                if name == "move" {
                    snake_acc.push(format_ident!("r#{}", name.to_snake_case()));
                } else {
                    snake_acc.push(format_ident!("{}", name.to_snake_case()));
                }

                plain_acc.push(name);

                ids.push(i);
                (ids, plain_acc, camel_acc, snake_acc)
            },
        );

    quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Grammar {
            #(#camel),*
        }

        impl Grammar {
            pub const VARIANTS: &[Self] = &[
                #(Self::#camel),*
            ];

            pub const fn highlight_configuration_params(self) -> crate::HighlightConfigurationParams {
                match self {
                    #(Self::#camel => crate::HighlightConfigurationParams {
                        language: crate::grammar::#snake::LANGUAGE,
                        name: #plain,
                        highlights_query: crate::grammar::#snake::HIGHLIGHTS_QUERY,
                        injection_query: crate::grammar::#snake::INJECTIONS_QUERY,
                        locals_query: crate::grammar::#snake::LOCALS_QUERY,
                    }),*
                }
            }

            pub const fn idx(self) -> usize {
                match self {
                    #(Self::#camel => #ids),*
                }
            }
        }
    }
}

fn build_language_module(
    name: &str,
    query_path: &Path,
) -> anyhow::Result<Option<proc_macro2::TokenStream>> {
    if is_blacklisted(name) {
        return Ok(None);
    }

    let highlights_query = read_local_query(query_path, name, "highlights.scm");
    let injections_query = read_local_query(query_path, name, "injections.scm");
    let locals_query = read_local_query(query_path, name, "locals.scm");

    let ffi = format_ident!("tree_sitter_{}", name.to_snake_case());
    let name = if name == "move" {
        format_ident!("r#{}", name.to_snake_case())
    } else {
        format_ident!("{}", name.to_snake_case())
    };

    Ok(Some(quote! {
        pub mod #name {
            extern "C" {
                fn #ffi() -> *const ();
            }

            pub const LANGUAGE: tree_sitter_language::LanguageFn = unsafe { tree_sitter_language::LanguageFn::from_raw(#ffi) };
            pub const HIGHLIGHTS_QUERY: &str = #highlights_query;
            pub const INJECTIONS_QUERY: &str = #injections_query;
            pub const LOCALS_QUERY: &str = #locals_query;
        }
    }))
}

// taken from https://github.com/helix-editor/helix/blob/2ce4c6d5fa3e50464b41a3d0190ad0e5ada2fc3c/helix-core/src/syntax.rs#L721
fn read_local_query(query_path: &Path, language: &str, filename: &str) -> String {
    static INHERITS_REGEX: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r";+\s*inherits\s*:?\s*([a-z_,()-]+)\s*").unwrap());

    let path = query_path.join(language).join(filename);

    if !path.exists() {
        return String::new();
    }

    let query =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to fetch {path:?}: {e:?}"));

    if filename == "injections.scm" {
        query
    } else {
        INHERITS_REGEX
            .replace_all(&query, |captures: &regex::Captures| {
                captures[1]
                    .split(',')
                    .fold(String::new(), |mut output, language| {
                        // `write!` to a String cannot fail.
                        write!(
                            output,
                            "\n{}\n",
                            read_local_query(query_path, language, filename)
                        )
                        .unwrap();
                        output
                    })
            })
            .to_string()
    }
}

fn fetch_and_build_grammar(
    grammars: Vec<GrammarDefinition>,
    source_dir: &Path,
    build_root: &Path,
) -> anyhow::Result<()> {
    let pool = ThreadPool::new(std::thread::available_parallelism()?.get());

    for grammar in grammars {
        if is_blacklisted(&grammar.name) {
            continue;
        }

        let grammar_root = source_dir.join(&grammar.name);
        let grammar_build_dir = grammar_cache_dir(build_root, &grammar)?;

        pool.execute(move || {
            let grammar_root = match &grammar.source {
                GrammarSource::Git {
                    remote,
                    revision,
                    subpath,
                } => {
                    fetch_git_repository(remote, revision, &grammar_root)
                        .context(GRAMMAR_REPOSITORY_URL)
                        .expect("failed to fetch git repository");

                    let mut grammar_root = grammar_root;
                    if let Some(subpath) = subpath {
                        grammar_root.push(subpath);
                    }

                    grammar_root
                }
                GrammarSource::Local { path } => path.clone(),
            };

            let grammar_src = grammar_root.join("src");

            let parser_file = Some(grammar_src.join("parser.c"))
                .filter(|s| s.exists())
                .or_else(|| Some(grammar_src.join("parser.cc")))
                .filter(|s| s.exists());
            let _scanner_file = Some(grammar_src.join("scanner.c"))
                .filter(|s| s.exists())
                .or_else(|| Some(grammar_src.join("scanner.cc")))
                .filter(|s| s.exists());

            // Handle scanner file:
            let mut actual_scanner_file = None;
            let scanner_c_path = grammar_src.join("scanner.c");
            let scanner_cc_path = grammar_src.join("scanner.cc");

            if scanner_c_path.exists() {
                actual_scanner_file = Some(scanner_c_path);
            } else if scanner_cc_path.exists() {
                actual_scanner_file = Some(scanner_cc_path);
            }

            // If a parser exists but no scanner file was found, create a dummy scanner file.
            if grammar.name == "groovy" || (parser_file.is_some() && actual_scanner_file.is_none()) {
                let dummy_scanner_path = grammar_src.join("scanner.c"); // Always create a .c scanner
                let snake_case_name = grammar.name.to_snake_case();
                let create_fn = format!("tree_sitter_{}_external_scanner_create", snake_case_name);
                let scan_fn = format!("tree_sitter_{}_external_scanner_scan", snake_case_name);
                let serialize_fn = format!("tree_sitter_{}_external_scanner_serialize", snake_case_name);
                let deserialize_fn = format!("tree_sitter_{}_external_scanner_deserialize", snake_case_name);
                let destroy_fn = format!("tree_sitter_{}_external_scanner_destroy", snake_case_name);

                let dummy_c_scanner = format!(
r#"
#include "tree_sitter/parser.h"

// Dummy external scanner functions to provide global symbols for grammar: {}
// This is a placeholder to prevent 'ranlib: archive library: ... the table of contents is empty' warning
// if the actual grammar does not provide an external scanner.

void *{create_fn}() {{
    return NULL;
}}

bool {scan_fn}(void *payload, TSLexer *lexer, const bool *valid_symbols) {{
    return false;
}}

unsigned {serialize_fn}(void *payload, char *buffer) {{
    return 0;
}}

void {deserialize_fn}(void *payload, const char *buffer, unsigned length) {{
}}

void {destroy_fn}(void *payload) {{
}}
"#,
                    grammar.name,
                    create_fn = create_fn,
                    scan_fn = scan_fn,
                    serialize_fn = serialize_fn,
                    deserialize_fn = deserialize_fn,
                    destroy_fn = destroy_fn
                );

                fs::write(&dummy_scanner_path, dummy_c_scanner)
                    .with_context(|| format!("failed to write dummy scanner to {dummy_scanner_path:?}"))
                    .unwrap();
                actual_scanner_file = Some(dummy_scanner_path); // Update scanner_file to point to our new dummy
                //println!("cargo:warning=DEBUG: Injected dummy scanner for {}.", grammar.name);
            }

            if grammar_artifacts_ready(
                &grammar_build_dir,
                &grammar,
                parser_file.is_some(),
                actual_scanner_file.is_some(),
            ) {
                emit_cached_grammar_links(
                    &grammar_build_dir,
                    &grammar,
                    parser_file.is_some(),
                    actual_scanner_file.is_some(),
                );
                return;
            }

            fs::create_dir_all(&grammar_build_dir).unwrap();

            if let Some(parser_file) = parser_file {
                cc::Build::new()
                    .out_dir(&grammar_build_dir)
                    .cpp(parser_file.extension() == Some(OsStr::new("cc")))
                    .file(parser_file)
                    .flag_if_supported("-w")
                    .flag_if_supported("-s")
                    .include(&grammar_src)
                    .compile(&format!("{}-parser", grammar.name));
            }

            if let Some(scanner_file_path) = actual_scanner_file {
                //println!("cargo:warning=DEBUG: Compiling scanner for {}: path={:?}", grammar.name, &scanner_file_path);
                cc::Build::new()
                    .out_dir(&grammar_build_dir)
                    .cpp(scanner_file_path.extension() == Some(OsStr::new("cc")))
                    .file(&scanner_file_path)
                    .flag_if_supported("-w")
                    .flag_if_supported("-s")
                    .include(&grammar_src)
                    .compile(&format!("{}-scanner", grammar.name));
            }

            emit_cache_marker(&grammar_build_dir, &grammar).unwrap();
        });
    }

    pool.join();

    Ok(())
}

fn grammar_cache_dir(build_root: &Path, grammar: &GrammarDefinition) -> anyhow::Result<PathBuf> {
    let mut hasher = DefaultHasher::new();
    grammar.name.hash(&mut hasher);
    grammar.source.hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());
    Ok(build_root.join(&grammar.name).join(key))
}

fn grammar_artifacts_ready(
    build_dir: &Path,
    grammar: &GrammarDefinition,
    has_parser: bool,
    has_scanner: bool,
) -> bool {
    build_dir.join("cache.marker").exists()
        && (!has_parser || artifact_exists(build_dir, &format!("{}-parser", grammar.name)))
        && (!has_scanner || artifact_exists(build_dir, &format!("{}-scanner", grammar.name)))
}

fn artifact_exists(build_dir: &Path, stem: &str) -> bool {
    let unix = format!("lib{stem}.a");
    let windows = format!("{stem}.lib");

    fs::read_dir(build_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name == unix || name == windows)
}

fn emit_cached_grammar_links(
    build_dir: &Path,
    grammar: &GrammarDefinition,
    has_parser: bool,
    has_scanner: bool,
) {
    println!("cargo::rustc-link-search=native={}", build_dir.display());
    if has_parser {
        println!("cargo::rustc-link-lib=static={}-parser", grammar.name);
    }
    if has_scanner {
        println!("cargo::rustc-link-lib=static={}-scanner", grammar.name);
    }
}

fn emit_cache_marker(build_dir: &Path, grammar: &GrammarDefinition) -> anyhow::Result<()> {
    fs::write(build_dir.join("cache.marker"), grammar.name.as_bytes())?;
    Ok(())
}

fn fetch_git_repository(url: &str, ref_: &str, destination: &Path) -> anyhow::Result<()> {
    if !destination.exists() {
        let res = Command::new("git").arg("init").arg(destination).status()?;
        if !res.success() {
            bail!("git init failed with exit code {res}");
        }

        let res = Command::new("git")
            .args(["remote", "add", "origin", url])
            .current_dir(destination)
            .status()?;
        if !res.success() {
            bail!("git remote failed with exit code {res}");
        }
    }

    let res = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(destination)
        .output()?
        .stdout;
    if res == ref_.as_bytes() {
        return Ok(());
    }

    let res = Command::new("git")
        .args(["fetch", "--depth", "1", "origin", ref_])
        .current_dir(destination)
        .status()?;
    if !res.success() {
        bail!("git fetch failed with exit code {res}");
    }

    let res = Command::new("git")
        .args(["reset", "--hard", ref_])
        .current_dir(destination)
        .status()?;
    if !res.success() {
        bail!("git fetch failed with exit code {res}");
    }

    Ok(())
}

fn fetch_helix_archive(destination: &Path) -> anyhow::Result<()> {
    let archive_url = format!(
        "https://codeload.github.com/helix-editor/helix/tar.gz/{}",
        GRAMMAR_REPOSITORY_REF
    );
    let temp_root = std::env::temp_dir().join(format!(
        "gnostr-grammar-helix-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    fs::create_dir_all(&temp_root)?;

    let archive_path = temp_root.join("helix.tar.gz");
    let download = Command::new("curl")
        .args(["-fsSL", &archive_url, "-o"])
        .arg(&archive_path)
        .status()?;
    if !download.success() {
        bail!("curl failed with exit code {download}");
    }

    fs::create_dir_all(destination)?;

    let extract = Command::new("tar")
        .args([
            "-xzf",
            archive_path
                .to_str()
                .context("archive path is not valid UTF-8")?,
            "-C",
            destination
                .to_str()
                .context("destination path is not valid UTF-8")?,
            "--strip-components=1",
        ])
        .status()?;
    if !extract.success() {
        bail!("tar extract failed with exit code {extract}");
    }

    let _ = fs::remove_dir_all(&temp_root);
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct LanguageDefinition {
    name: String,
    injection_regex: Option<String>,
    file_types: Vec<FileType>,
    grammar: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum FileType {
    Glob { glob: String },
    Extension(String),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GrammarDefinition {
    name: String,
    source: GrammarSource,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase", untagged)]
enum GrammarSource {
    Git {
        #[serde(rename = "git")]
        remote: String,
        #[serde(rename = "rev")]
        revision: String,
        subpath: Option<String>,
    },
    Local {
        path: PathBuf,
    },
}

impl Hash for GrammarSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            GrammarSource::Git {
                remote,
                revision,
                subpath,
            } => {
                "git".hash(state);
                remote.hash(state);
                revision.hash(state);
                subpath.hash(state);
            }
            GrammarSource::Local { path } => {
                "local".hash(state);
                path.hash(state);
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HelixLanguages {
    language: Vec<LanguageDefinition>,
    grammar: Vec<GrammarDefinition>,
}
