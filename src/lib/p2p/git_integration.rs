use git2::{Commit, DiffFormat, ObjectType, Repository};

pub fn get_commit_diff_as_bytes(
    repo: &Repository,
    commit: &Commit,
) -> Result<Vec<u8>, git2::Error> {
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let mut buf = Vec::new();

    diff.print(DiffFormat::Patch, |_, _, line| {
        buf.extend_from_slice(line.content());
        true
    })?;

    Ok(buf)
}

pub fn get_commit_id_of_tag(repo: &Repository, tag_name: &str) -> Result<String, git2::Error> {
    let reference_name = format!("refs/tags/{}", tag_name);
    let reference = repo.find_reference(&reference_name)?;
    let object = reference.peel(ObjectType::Commit)?;
    Ok(object.id().to_string())
}
