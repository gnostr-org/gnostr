pub use gitui_git2_hooks::*;

#[cfg(test)]
mod tests {
    use git2_testing::{repo_init, repo_init_bare};
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_smoke() {
        let (_td, repo) = repo_init();

        let mut msg = String::from("test");
        let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

        assert_eq!(res, HookResult::NoHookFound);

        let hook = b"#!/bin/sh
exit 0
        ";

        create_hook(&repo, HOOK_POST_COMMIT, hook);

        let res = hooks_post_commit(&repo, None).unwrap();

        assert!(res.is_ok());
    }

    #[test]
    fn test_hooks_commit_msg_ok() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
exit 0
        ";

        create_hook(&repo, HOOK_COMMIT_MSG, hook);

        let mut msg = String::from("test");
        let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

        assert!(res.is_ok());

        assert_eq!(msg, String::from("test"));
    }

    #[test]
    fn test_hooks_commit_msg_with_shell_command_ok() {
        let (_td, repo) = repo_init();

        let hook = br#"#!/bin/sh
COMMIT_MSG="$(cat "$1")"
printf "$COMMIT_MSG" | sed 's/sth/shell_command/g' >"$1"
exit 0
        "#;

        create_hook(&repo, HOOK_COMMIT_MSG, hook);

        let mut msg = String::from("test_sth");
        let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

        assert!(res.is_ok());

        assert_eq!(msg, String::from("test_shell_command"));
    }

    #[test]
    fn test_pre_commit_sh() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
exit 0
        ";

        create_hook(&repo, HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(&repo, None).unwrap();
        assert!(res.is_ok());
    }

    #[test]
    fn test_no_hook_found() {
        let (_td, repo) = repo_init();

        let res = hooks_pre_commit(&repo, None).unwrap();
        assert_eq!(res, HookResult::NoHookFound);
    }

    #[test]
    fn test_other_path() {
        let (td, repo) = repo_init();

        let hook = b"#!/bin/sh
exit 0
        ";

        let custom_hooks_path = td.path().join(".myhooks");

        std::fs::create_dir(dbg!(&custom_hooks_path)).unwrap();
        create_hook_in_path(
            dbg!(custom_hooks_path.join(HOOK_PRE_COMMIT).as_path()),
            hook,
        );

        let res = hooks_pre_commit(&repo, Some(&["../.myhooks"])).unwrap();

        assert!(res.is_ok());
    }

    #[test]
    fn test_other_path_precendence() {
        let (td, repo) = repo_init();

        {
            let hook = b"#!/bin/sh
exit 0
        ";

            create_hook(&repo, HOOK_PRE_COMMIT, hook);
        }

        {
            let reject_hook = b"#!/bin/sh
exit 1
        ";

            let custom_hooks_path = td.path().join(".myhooks");
            std::fs::create_dir(dbg!(&custom_hooks_path)).unwrap();
            create_hook_in_path(
                dbg!(custom_hooks_path.join(HOOK_PRE_COMMIT).as_path()),
                reject_hook,
            );
        }

        let res = hooks_pre_commit(&repo, Some(&["../.myhooks"])).unwrap();

        assert!(res.is_ok());
    }

    #[test]
    fn test_pre_commit_fail_sh() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
echo 'rejected'
exit 1
        ";

        create_hook(&repo, HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(&repo, None).unwrap();
        assert!(res.is_not_successful());
    }

    #[test]
    fn test_env_containing_path() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
export
exit 1
        ";

        create_hook(&repo, HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(&repo, None).unwrap();

        let HookResult::RunNotSuccessful { stdout, .. } = res else {
            unreachable!()
        };

        assert!(stdout.lines().any(|line| line.starts_with("export PATH")));
    }

    #[test]
    fn test_pre_commit_fail_hookspath() {
        let (_td, repo) = repo_init();
        let hooks = TempDir::new().unwrap();

        let hook = b"#!/bin/sh
echo 'rejected'
exit 1
        ";

        create_hook_in_path(&hooks.path().join("pre-commit"), hook);

        repo.config()
            .unwrap()
            .set_str("core.hooksPath", hooks.path().as_os_str().to_str().unwrap())
            .unwrap();

        let res = hooks_pre_commit(&repo, None).unwrap();

        let HookResult::RunNotSuccessful { code, stdout, .. } = res else {
            unreachable!()
        };

        assert_eq!(code.unwrap(), 1);
        assert_eq!(&stdout, "rejected\n");
    }

    #[test]
    fn test_pre_commit_fail_bare() {
        let (_td, repo) = repo_init_bare();

        let hook = b"#!/bin/sh
echo 'rejected'
exit 1
        ";

        create_hook(&repo, HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(&repo, None).unwrap();
        assert!(res.is_not_successful());
    }

    #[test]
    fn test_pre_commit_py() {
        let (_td, repo) = repo_init();

        // mirror how python pre-commit sets itself up
        #[cfg(not(windows))]
        let hook = b"#!/usr/bin/env python3
import sys
sys.exit(0)
        ";
        #[cfg(windows)]
        let hook = b"#!/bin/env python3.exe
import sys
sys.exit(0)
        ";

        create_hook(&repo, HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(&repo, None).unwrap();
        assert!(res.is_ok());
    }

    #[test]
    fn test_pre_commit_fail_py() {
        let (_td, repo) = repo_init();

        // mirror how python pre-commit sets itself up
        #[cfg(not(windows))]
        let hook = b"#!/usr/bin/env python3
import sys
sys.exit(1)
        ";
        #[cfg(windows)]
        let hook = b"#!/bin/env python3.exe
import sys
sys.exit(1)
        ";

        create_hook(&repo, HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(&repo, None).unwrap();
        assert!(res.is_not_successful());
    }

    #[test]
    fn test_hooks_commit_msg_reject() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
echo 'msg' > $1
echo 'rejected'
exit 1
        ";

        create_hook(&repo, HOOK_COMMIT_MSG, hook);

        let mut msg = String::from("test");
        let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

        let HookResult::RunNotSuccessful { code, stdout, .. } = res else {
            unreachable!()
        };

        assert_eq!(code.unwrap(), 1);
        assert_eq!(&stdout, "rejected\n");

        assert_eq!(msg, String::from("msg\n"));
    }

    #[test]
    fn test_commit_msg_no_block_but_alter() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
echo 'msg' > $1
exit 0
        ";

        create_hook(&repo, HOOK_COMMIT_MSG, hook);

        let mut msg = String::from("test");
        let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

        assert!(res.is_ok());
        assert_eq!(msg, String::from("msg\n"));
    }

    #[test]
    fn test_hook_pwd_in_bare_without_workdir() {
        let (_td, repo) = repo_init_bare();
        let git_root = repo.path().to_path_buf();

        let hook = HookPaths::new(&repo, None, HOOK_POST_COMMIT).unwrap();

        assert_eq!(hook.pwd, git_root);
    }

    #[test]
    fn test_hook_pwd() {
        let (_td, repo) = repo_init();
        let git_root = repo.path().to_path_buf();

        let hook = HookPaths::new(&repo, None, HOOK_POST_COMMIT).unwrap();

        assert_eq!(hook.pwd, git_root.parent().unwrap());
    }

    #[test]
    fn test_hooks_prep_commit_msg_success() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
echo msg:$2 > $1
exit 0
        ";

        create_hook(&repo, HOOK_PREPARE_COMMIT_MSG, hook);

        let mut msg = String::from("test");
        let res = hooks_prepare_commit_msg(&repo, None, PrepareCommitMsgSource::Message, &mut msg)
            .unwrap();

        assert!(matches!(res, HookResult::Ok { .. }));
        assert_eq!(msg, String::from("msg:message\n"));
    }

    #[test]
    fn test_hooks_prep_commit_msg_reject() {
        let (_td, repo) = repo_init();

        let hook = b"#!/bin/sh
echo $2,$3 > $1
echo 'rejected'
exit 2
        ";

        create_hook(&repo, HOOK_PREPARE_COMMIT_MSG, hook);

        let mut msg = String::from("test");
        let res = hooks_prepare_commit_msg(
            &repo,
            None,
            PrepareCommitMsgSource::Commit(git2::Oid::zero()),
            &mut msg,
        )
        .unwrap();

        let HookResult::RunNotSuccessful { code, stdout, .. } = res else {
            unreachable!()
        };

        assert_eq!(code.unwrap(), 2);
        assert_eq!(&stdout, "rejected\n");

        assert_eq!(
            msg,
            String::from("commit,0000000000000000000000000000000000000000\n")
        );
    }
}
