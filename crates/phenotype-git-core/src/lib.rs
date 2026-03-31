//! phenotype-git-core

use git2::Repository;
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
    pub fn open(path: &Path) -> Result<Self, git2::Error> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn is_bare(&self) -> bool {
        self.repo.is_bare()
    }

    pub fn head_commit(&self) -> Result<Option<GitCommit>, git2::Error> {
        let head = self.repo.head();
        match head {
            Ok(head) => {
                let oid = head.target().ok_or_else(|| {
                    git2::Error::new(
                        git2::ErrorCode::Invalid,
                        git2::ErrorLevel::Warning,
                        "head has no target",
                    )
                })?;
                let commit = self.repo.find_commit(oid)?;
                Ok(Some(GitCommit {
                    id: oid.to_string()[..8].to_string(),
                    message: commit.message().unwrap_or("").to_string(),
                }))
            }
            Err(e) => {
                if e.code() == git2::ErrorCode::UnbornBranch
                    || e.code() == git2::ErrorCode::NotFound
                {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }
}
