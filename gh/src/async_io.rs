use std::{fmt, io};
use tokio::io::{stdout, stdin, BufReader, AsyncWriteExt, AsyncBufReadExt};
pub use tokio::fs;
pub use regex::Regex;
pub use async_std::path as path;
use std::path as std_path;

pub async fn _aprint_args(args: fmt::Arguments<'_>) {
    let mut to_write = String::new();
    let _ = match fmt::write(&mut to_write, args) {
        Ok(r) => r,
        Err(e) => panic!("failed format, error: {e}"),
    };
    let mut out = stdout();
    let _ = match out.write(to_write.as_bytes()).await {
        Ok(r) => r,
        Err(e) => panic!("failed printing to stdout, error: {e}"),
    };
    let _ = match out.flush().await {
        Ok(r) => r,
        Err(e) => panic!("failed flush stdout, error: {e}"),
    };
}

#[macro_export]
macro_rules! aprint {
    ($($arg:tt)*) => (_aprint_args(format_args!($($arg)*)).await)
}

#[macro_export]
macro_rules! aprintln {
    () => (aprint!("\n"));
    ($($arg:tt)*) => ({
        aprint!($($arg)*);
        aprint!("\n");
    })
}

#[macro_export]
macro_rules! fn_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(f)
            .trim_end_matches("::f")
            .replace("::{{closure}}", "")
    }}
}

#[macro_export]
macro_rules! adprintln {
    () => (aprintln!("{fn}:{line}", fn = fn_name!(), line = line!()));
    ($($arg:tt)*) => (async {
        aprint!("{fn}:{line}: ", fn = fn_name!(), line = line!());
        aprintln!($($arg)*);
    }.await)
}

#[macro_export]
macro_rules! dprintln {
    () => (println!("{fn}:{line}", fn = fn_name!(), line = line!()));
    ($($arg:tt)*) => ({
        print!("{fn}:{line}: ", fn = fn_name!(), line = line!());
        println!($($arg)*);
    })
}

pub fn path_display_sync(path: &std_path::PathBuf) -> String {
    path.to_str().unwrap_or(path.to_string_lossy().to_string().as_str()).to_owned()
}

pub fn path_display(path: &path::PathBuf) -> String {
    path.to_str().unwrap_or(path.to_string_lossy().to_string().as_str()).to_owned()
}

pub async fn path_args(args: fmt::Arguments<'_>) -> path::PathBuf {
    let mut path_string = String::new();
    let _ = match fmt::write(&mut path_string, args) {
        Ok(r) => r,
        Err(e) => panic!("Failed to format, error: {e}"),
    };
    let path: path::PathBuf;
    if path_string.starts_with("~") {
        let home_path = match home::home_dir() {
            Some(pb) => path::PathBuf::from(pb),
            None => path::PathBuf::from("."),
        };
        let home_path_abs = match home_path.canonicalize().await {
            Ok(p) => p,
            Err(e) => panic!(
                "Failed to canonicalize home path \"{home_path_str}\" error: {e}",
                home_path_str = path_display(&home_path)
            ),
        };
        path_string = path_string[1..].to_string();
        path = match home_path_abs.to_str() {
            Some(home_path_abs_str) => {
                path::PathBuf::from(format!("{home_path_abs_str}{path_string}"))
            },
            None => {
                let path_string_no_prefix = path_string.strip_prefix("/").unwrap_or(&path_string);
                home_path_abs.join(path_string_no_prefix)
            }
        };
    } else {
        path = path::PathBuf::from(path_string); 
    }
    path
}

#[macro_export]
macro_rules! path {
    ($($arg:tt)*) => (path_args(format_args!($($arg)*)).await)
}

pub fn path_to_sync(path: &path::PathBuf) -> std_path::PathBuf {
    std_path::PathBuf::from(
        path.to_str().expect("can encode path")
    )
}

pub fn path_rel_sync(parent: &std_path::Path, child: &std_path::Path) -> io::Result<std_path::PathBuf> {
    // This routine is adapted from the *old* Path's `path_relative_from`
    // function, which works differently from the new `relative_from` function.
    // In particular, this handles the case on unix where both paths are
    // absolute but with only the root as the common directory.
    use std::path::Component;
    if !parent.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, 
            format!(
                "Parent path ({parent_display}) must be absolute",
                parent_display = path_display_sync(&(parent.to_path_buf())),
            )
        ));
    }
    if !child.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, 
            format!(
                "Child path ({child_display}) must be absolute",
                child_display = path_display_sync(&(child.to_path_buf())),
            )
        ));
    }

    let mut it_child = child.components();
    let mut it_parent = parent.components();
    let mut comps: Vec<Component> = vec![];
    loop {
        match (it_child.next(), it_parent.next()) {
            (None, None) => break,
            (Some(c), None) => {
                comps.push(c);
                comps.extend(it_child.by_ref());
                break;
            }
            (None, _) => comps.push(Component::ParentDir),
            (Some(c), Some(p)) if comps.is_empty() && c == p => (),
            (Some(c), Some(p)) if p == Component::CurDir => comps.push(c),
            (Some(_), Some(p)) if p == Component::ParentDir => return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Parent path ({parent_display}) does not contain child path ({child_display})",
                    parent_display = path_display_sync(&(parent.to_path_buf())),
                    child_display = path_display_sync(&(child.to_path_buf())),
                )
            )),
            (Some(c), Some(_)) => {
                comps.push(Component::ParentDir);
                for _ in it_parent {
                    comps.push(Component::ParentDir);
                }
                comps.push(c);
                comps.extend(it_child.by_ref());
                break;
            }
        }
    }
    Ok(comps.iter().map(|c| c.as_os_str()).collect())
}

pub async fn stdin_read_line() -> String {
    let mut line = String::new();
    BufReader::new(stdin()).read_line(&mut line).await.expect("can read line");
    line
}

pub async fn prompt(text: &str, default: Option<&str>) -> String {
    let _ = match default {
        Some(default_verbose) => aprint!("{text} [{default_verbose}]: "),
        None => aprint!("{text}: "),
    };
    let answer = stdin_read_line().await;
    if let Some(default_verbose) = default { 
        if answer.is_empty() {
            return default_verbose.to_string();
        }
    }
    answer
}

pub async fn prompt_bool(text: &str, default: Option<bool>) -> bool {
    loop {
        let _ = match default {
            Some(default_bool) => {
                let default_verbose = if default_bool {"yes"} else {"no"};
                aprint!("{text} y(es) / n(o) [{default_verbose}]: ")
            },
            None => aprint!("{text} y(es) / n(o): "),
        };
        return match stdin_read_line().await.to_lowercase().trim() {
            "y"|"yes" => true,
            "n"|"no" => false,
            "" => match default {
                Some(default_bool) => default_bool,
                None => {
                    aprintln!("Invalid answer, type one of: \"yes\", \"y\", \"no\", \"n\" (in any case)");
                    continue;
                },
            },
            _ => { 
                aprintln!("Invalid answer, type one of: \"yes\", \"y\", \"no\", \"n\" (in any case)");
                continue;
            },
        }
    }
}

pub fn re(regex_str: &str) -> Regex { Regex::new(regex_str).expect("Valid regex") }

pub fn regex_groups<'string>(regex: &Regex, string: &'string str) -> Vec<regex::Match<'string>> {
    let mut groups: Vec<regex::Match<'string>> = vec![];
    for captures in regex.captures_iter(string) {
        for group_opt in captures.iter() {
            if let Some(group) = group_opt {
                groups.push(group);
            }
        }
    }
    groups
}

pub async fn find_regex_in_file_lines(
    file_path: &path::PathBuf, 
    regex: Regex, 
    group_i: usize
) -> Option<String> {
    if !file_path.is_file().await { return None; }
    let file = match fs::File::open(file_path).await {
        Ok(f) => f,
        Err(e) => return None,
    };
    let mut reader = BufReader::new(file);
    let mut i = 0;
    loop {
        let mut buffer = String::new();
        let line = match reader
            .read_line(&mut buffer)
            .await
            .map(|u| if u == 0 { None } else { Some(buffer) })
            .transpose()
        {
            Some(Ok(l)) => l,
            Some(Err(e)) => { continue; },
            None => { break; }
        };
        for group in regex_groups(&regex, line.as_str()) {
            if i != group_i { return Some(String::from(group.as_str())); }
            i += 1;
        }
    }
    None
}

pub async fn read_dir_filenames(path: impl AsRef<std_path::Path>) -> Vec<String> {
    let mut read_dir = fs::read_dir(path).await.expect("can read dir");
    let mut filepaths = vec![];
    while let Some(entry) = read_dir.next_entry().await.expect("can get next DirEntry") {
        filepaths.push(entry.file_name().into_string().expect("can convert to string"));
    }
    filepaths
}
