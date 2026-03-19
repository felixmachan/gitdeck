use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct RepoSummary {
    pub repo_name: String,
    pub branch: String,
    pub head_short: String,
    pub remotes: Vec<String>,
    pub status: WorkingTreeStatus,
    pub operation: Option<RepoOperation>,
    pub detached_head: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkingTreeStatus {
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicts: usize,
}

impl WorkingTreeStatus {
    pub fn is_clean(&self) -> bool {
        self.staged == 0 && self.modified == 0 && self.untracked == 0 && self.conflicts == 0
    }

    pub fn label(&self) -> String {
        if self.is_clean() {
            "clean".to_string()
        } else {
            format!(
                "staged:{} modified:{} untracked:{} conflicts:{}",
                self.staged, self.modified, self.untracked, self.conflicts
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum RepoOperation {
    Merge,
    Rebase,
    CherryPick,
}

#[derive(Debug, Clone)]
pub struct CommitSummary {
    pub id: String,
    pub short_id: String,
    pub author: String,
    pub date: DateTime<Local>,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct CommitDetails {
    pub summary: CommitSummary,
    pub body: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub is_head: bool,
    pub upstream: Option<String>,
}
