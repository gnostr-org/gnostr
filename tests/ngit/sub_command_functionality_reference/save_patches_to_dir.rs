use anyhow::Result;
use test_utils::git::GitTestRepo;
use crate::lib::sub_commands::list::save_patches_to_dir;
use nostr_0_34_1::{Event, EventId, Kind, Keys, Tag, Timestamp};
use std::fs;
use std::path::PathBuf;
use hex::FromHex;

fn create_mock_patch_event(content: &str, commit_msg: &str, parent_commit_id: &str) -> Event {
    let keys = Keys::generate();
    Event::new(
        &keys,
        Kind::Custom(30000), // Git Patch kind
        vec![
            Tag::parse(["description", commit_msg]).unwrap(),
            Tag::parse(["parent-commit", parent_commit_id]).unwrap(),
            Tag::parse(["commit", &EventId::from_hex("5e0b5a7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e").unwrap().to_string()]).unwrap(),
        ],
        content,
        Timestamp::now(),
    ).unwrap()
}

#[tokio::test]
#[serial]
async fn test_save_patches_to_dir() -> Result<()> {
    let test_repo = GitTestRepo::default();
    test_repo.populate()?;

    let patch_content_1 = "From 5e0b5a7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e Mon Sep 17 00:00:00 2001\nFrom: Test User <test@example.com>\nDate: Mon, 1 Jan 2024 00:00:00 +0000\nSubject: [PATCH] First commit\n\n---\n newfile.txt | 1 +\n 1 file changed, 1 insertion(+)\n\ndiff --git a/newfile.txt b/newfile.txt\nnew file mode 100644\nindex 0000000..e69de29\n--- /dev/null\n+++ b/newfile.txt\n@@ -0,0 +1 @@\n+Hello, world!\n\\ No newline at end of file\n---";
    let patch_content_2 = "From 6f1c6b8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f Mon Sep 17 00:00:00 2001\nFrom: Test User <test@example.com>\nDate: Mon, 1 Jan 2024 00:00:01 +0000\nSubject: [PATCH] Second commit\n\n---\n another.txt | 1 +\n 1 file changed, 1 insertion(+)\n\ndiff --git a/another.txt b/another.txt\nnew file mode 100644\nindex 0000000..e69de29\n--- /dev/null\n+++ b/another.txt\n@@ -0,0 +1 @@\n+Another file\n\\ No newline at end of file\n---";

    let patch_1 = create_mock_patch_event(patch_content_1, "First commit", "0000000000000000000000000000000000000000");
    let patch_2 = create_mock_patch_event(patch_content_2, "Second commit", "5e0b5a7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e7e");

    let patches = vec![patch_1.clone(), patch_2.clone()];

    save_patches_to_dir(patches, &test_repo.repo)?;

    let patches_dir = test_repo.dir.join("patches");
    assert!(patches_dir.exists());
    assert!(patches_dir.is_dir());

    let files: Vec<PathBuf> = fs::read_dir(&patches_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    assert_eq!(files.len(), 2);

    // Verify first patch file
    let expected_file_name_1 = format!("{}-{:0>4}-{}.patch", &patch_1.id.to_string()[..5], 1, "First-commit");
    let file_path_1 = patches_dir.join(&expected_file_name_1);
    assert!(file_path_1.exists());
    assert_eq!(fs::read_to_string(&file_path_1)?, format!("{}\n\n", patch_content_1));

    // Verify second patch file
    let expected_file_name_2 = format!("{}-{:0>4}-{}.patch", &patch_1.id.to_string()[..5], 2, "Second-commit");
    let file_path_2 = patches_dir.join(&expected_file_name_2);
    assert!(file_path_2.exists());
    assert_eq!(fs::read_to_string(&file_path_2)?, format!("{}\n\n", patch_content_2));

    Ok(())
}
