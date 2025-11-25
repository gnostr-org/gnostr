use gnostr_filetreelist::{FileTree, MoveSelection};
use std::{collections::BTreeSet, path::Path, thread, time::Duration};

fn print_tree(tree: &FileTree) {
    println!("\n------------- Tree View -------------");
    if tree.is_empty() {
        println!("Empty");
    } else {
        for (item, selected) in tree.iterate(0, 100) {
            let indent = "  ".repeat(item.info().indent() as usize);
            let selection_marker = if selected { ">" } else { " " };

            let path_display = item.info().path().to_string_lossy();

            let kind_marker = if item.kind().is_path() {
                if item.kind().is_path_collapsed() {
                    "+"
                } else {
                    "-"
                }
            } else {
                ""
            };

            println!(
                "{} {}{}{} {}",
                selection_marker,
                indent,
                path_display,
                if item.kind().is_path() { "/" } else { "" },
                kind_marker
            );
        }
    }
    println!("-------------------------------------\n");
}

fn wait() {
    thread::sleep(Duration::from_secs(1));
}

fn main() {
    let paths: &[&Path] = &[
        Path::new("src/main.rs"),
        Path::new("src/lib.rs"),
        Path::new("src/components/a.rs"),
        Path::new("src/components/b.rs"),
        Path::new("README.md"),
        Path::new("LICENSE"),
        Path::new("examples/demo.rs"),
    ];

    let mut collapsed = BTreeSet::new();
    let src_components = "src/components".to_string();
    collapsed.insert(&src_components);

    let mut tree = FileTree::new(paths, &collapsed).unwrap();

    println!("Initial tree (with 'src/components' collapsed):");
    print_tree(&tree);
    wait();

    println!("Action: MoveSelection::Down");
    tree.move_selection(MoveSelection::Down);
    print_tree(&tree);
    wait();

    println!("Action: MoveSelection::Down");
    tree.move_selection(MoveSelection::Down);
    print_tree(&tree);
    wait();

    println!("Action: MoveSelection::Right (expand 'src/components')");
    tree.move_selection(MoveSelection::Right);
    print_tree(&tree);
    wait();

    println!("Action: MoveSelection::Down");
    tree.move_selection(MoveSelection::Down);
    print_tree(&tree);
    wait();

    println!("Action: MoveSelection::Left (collapse parent 'components')");
    tree.move_selection(MoveSelection::Left);
    print_tree(&tree);
    wait();

    println!("Action: MoveSelection::Left (collapse parent 'src')");
    tree.move_selection(MoveSelection::Left);
    print_tree(&tree);
    wait();

    println!("Action: Selecting 'src/main.rs' directly");
    tree.select_file(Path::new("src/main.rs"));
    print_tree(&tree);
    wait();

    println!("Action: Expand selection recursively");
    tree.expand_recursive();
    print_tree(&tree);
    wait();

    println!("Action: Collapse selection recursively");
    tree.collapse_recursive();
    print_tree(&tree);
    wait();

    println!("Demo finished.");
}
