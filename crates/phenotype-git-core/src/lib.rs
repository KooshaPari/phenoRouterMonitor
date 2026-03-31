//! phenotype-git-core

use gix::Repository;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub id: String,
    pub message: String,
}

impl GitCommit {
    pub fn new(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            message: message.to_string(),
        }
    }
}

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open(path: &Path) -> Result<Self, Box<gix::open::Error>> {
        let repo = gix::open(path).map_err(Box::new)?;
        Ok(Self { repo })
    }

    pub fn is_bare(&self) -> bool {
        self.repo.is_bare()
    }

    pub fn head_commit(&self) -> Result<Option<GitCommit>, gix::reference::head_commit::Error> {
        match self.repo.head_commit() {
            Ok(commit) => {
                let oid = commit.id();
                let message = commit
                    .message()
                    .unwrap_or_else(|_| gix::objs::commit::MessageRef {
                        title: b"<invalid>".as_ref().into(),
                        body: None,
                    });
                let message_str = if let Some(body) = message.body {
                    format!("{}{}{}", message.title, "\n\n", body)
                } else {
                    message.title.to_string()
                };
                Ok(Some(GitCommit {
                    id: oid.to_string()[..8].to_string(),
                    message: message_str,
                }))
            }
            Err(_) => Ok(None),
        }
    }
}
