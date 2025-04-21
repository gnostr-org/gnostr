use git2::Signature;
use git2::ObjectType;

//use nostr_sdk_0_37_0::prelude::*;
use nostr_sdk::prelude::*;
use scopetime::scope_time;

use super::{CommitId, RepoPath, commits_info::get_message};
use crate::{error::Result, sync::repository::repo};

///
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct CommitSignature {
	///
	pub name: String,
	///
	pub email: String,
	/// time in secs since Unix epoch
	pub time: i64,
}

impl CommitSignature {
	/// convert from git2-rs `Signature`
	pub fn from(s: &Signature<'_>) -> Self {
		Self {
			name: s.name().unwrap_or("").to_string(),
			email: s.email().unwrap_or("").to_string(),

			time: s.when().seconds(),
		}
	}
}

///
#[derive(Default, Clone)]
pub struct CommitMessage {
	/// first line
	pub subject: String,
	/// remaining lines if more than one
	pub body: Option<String>,
}

impl CommitMessage {
	///
	pub fn from(s: &str) -> Self {
		let mut lines = s.lines();
		let subject = lines.next().map_or_else(
			String::new,
			std::string::ToString::to_string,
		);

		let body: Vec<String> =
			lines.map(std::string::ToString::to_string).collect();

		Self {
			subject,
			body: if body.is_empty() {
				None
			} else {
				Some(body.join("\n"))
			},
		}
	}

	///
	pub fn combine(self) -> String {
		if let Some(body) = self.body {
			format!("{}\n{body}", self.subject)
		} else {
			self.subject
		}
	}
}

///
#[derive(Default, Clone)]
pub struct CommitDetails {
	///
	pub author: CommitSignature,
	/// committer when differs to `author` otherwise None
	pub committer: Option<CommitSignature>,
	///
	pub message: Option<CommitMessage>,
	///
	pub hash: String,
	///
	pub keys: Option<Keys>,
}

impl CommitDetails {
	///
	pub fn short_hash(&self) -> &str {
		&self.hash[0..7]
	}
	///
	pub (crate) fn pad_commit_hash(input: &str, padding_char: char) -> String {
		let target_length = 64;
		let current_length = input.len();
		if current_length >= target_length {
			return input.to_string(); // No padding needed
		}
		let padding_needed = target_length - current_length;
		let padding = padding_char.to_string().repeat(padding_needed);
		format!("{}{}", input, padding)
    }
	///
	pub fn padded_hash(&self) -> String {
		Self::pad_commit_hash(&self.hash, '0')
		//format!("{:0>64}", "".to_owned() + &self.hash[0..])
	}
	///
	pub fn padded_short_hash(&self) -> String {
		Self::pad_commit_hash(&self.hash[0..7], '0')
	}
	///
	pub fn keys(&self) -> Result<Keys> {
        Ok(Keys::parse(Self::pad_commit_hash(&self.hash, '0')).unwrap())
    }
}

///
pub fn get_commit_details(
	repo_path: &RepoPath,
	id: CommitId,
) -> Result<CommitDetails> {
	scope_time!("get_commit_details");

	let repo = repo(repo_path)?;
    let head = repo.head()?;
    let obj = head.resolve()?.peel(ObjectType::Commit)?;

    //read top commit
    let commit = obj.peel_to_commit()?;
    let commit_id = commit.id().to_string();
    //some info wrangling
    //info!("commit_id:\n{}", commit_id);
    let padded_commit_id = format!("{:0>64}", commit_id);

    //// commit based keys
    //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    //info!("keys.public_key():\n{}", keys.public_key());

    //parse keys from sha256 hash
    let keys = Keys::parse(padded_commit_id).unwrap();



	let commit = repo.find_commit(id.into())?;

	let author = CommitSignature::from(&commit.author());
	let committer = CommitSignature::from(&commit.committer());
	let committer = if author == committer {
		None
	} else {
		Some(committer)
	};

	let msg =
		CommitMessage::from(get_message(&commit, None).as_str());

	let details = CommitDetails {
		author,
		committer,
		message: Some(msg),
		hash: id.to_string(),
		keys: Some(keys),
	};

	Ok(details)
}

#[cfg(test)]
mod tests {
	use std::{fs::File, io::Write, path::Path};

	use super::{CommitMessage, get_commit_details};
	use crate::{
		error::Result,
		sync::{
			RepoPath, commit, stage_add_file, tests::repo_init_empty,
		},
	};

	#[test]
	fn test_msg_invalid_utf8() -> Result<()> {
		let file_path = Path::new("foo");
		let (_td, repo) = repo_init_empty().unwrap();
		let root = repo.path().parent().unwrap();
		let repo_path: &RepoPath =
			&root.as_os_str().to_str().unwrap().into();

		File::create(root.join(file_path))?.write_all(b"a")?;
		stage_add_file(repo_path, file_path).unwrap();

		let msg = invalidstring::invalid_utf8("test msg");
		let id = commit(repo_path, msg.as_str()).unwrap();

		let res = get_commit_details(repo_path, id).unwrap();

		assert_eq!(
			res.message
				.as_ref()
				.unwrap()
				.subject
				.starts_with("test msg"),
			true
		);

		Ok(())
	}

	#[test]
	fn test_msg_linefeeds() -> Result<()> {
		let msg = CommitMessage::from("foo\nbar\r\ntest");

		assert_eq!(msg.subject, String::from("foo"),);
		assert_eq!(msg.body, Some(String::from("bar\ntest")),);

		Ok(())
	}

	#[test]
	fn test_commit_message_combine() -> Result<()> {
		let msg = CommitMessage::from("foo\nbar\r\ntest");

		assert_eq!(msg.combine(), String::from("foo\nbar\ntest"));

		Ok(())
	}
}
