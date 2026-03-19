use anyhow::{Context, Result};
use chrono::{Local, TimeZone};
use git2::{BranchType, Repository, Sort, Status, StatusOptions};
use std::collections::HashMap;
use std::path::Path;

use crate::models::domain::{
    BranchInfo, CommitDetails, CommitSummary, RepoOperation, RepoSummary, WorkingTreeStatus,
};
use crate::models::graph::{GraphData, GraphNode};

pub struct GitService {
    repo: Repository,
}

impl GitService {
    pub fn discover<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path).context("Not inside a git repository")?;
        Ok(Self { repo })
    }

    pub fn repo_summary(&self) -> Result<RepoSummary> {
        let head = self.repo.head().ok();
        let head_oid = head.as_ref().and_then(|h| h.target());
        let branch = head
            .as_ref()
            .and_then(|h| h.shorthand())
            .unwrap_or("detached")
            .to_string();

        let detached_head = self.repo.head_detached().unwrap_or(false);
        let remotes = self
            .repo
            .remotes()?
            .iter()
            .flatten()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        let repo_name = self
            .repo
            .workdir()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("repo")
            .to_string();

        let status = self.collect_status()?;
        let operation = self.operation_state();
        let upstream = self.current_upstream();

        Ok(RepoSummary {
            repo_name,
            branch,
            head_short: head_oid
                .map(|oid| format!("{oid:.8}"))
                .unwrap_or_else(|| "none".to_string()),
            remotes,
            status,
            operation,
            detached_head,
            upstream,
        })
    }

    pub fn commit_history(&self, limit: usize) -> Result<Vec<CommitSummary>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push_head()?;

        let mut commits = Vec::new();
        for oid_result in revwalk.take(limit) {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;
            let time = commit.time();
            let date = Local
                .timestamp_opt(time.seconds(), 0)
                .single()
                .unwrap_or_else(Local::now);

            commits.push(CommitSummary {
                id: oid.to_string(),
                short_id: format!("{oid:.8}"),
                author: commit.author().name().unwrap_or("unknown").to_string(),
                date,
                subject: commit.summary().unwrap_or("(no subject)").to_string(),
            });
        }

        Ok(commits)
    }

    pub fn commit_details(&self, oid: &str) -> Result<CommitDetails> {
        let oid = git2::Oid::from_str(oid)?;
        let commit = self.repo.find_commit(oid)?;
        let summary = CommitSummary {
            id: oid.to_string(),
            short_id: format!("{oid:.8}"),
            author: commit.author().name().unwrap_or("unknown").to_string(),
            date: Local
                .timestamp_opt(commit.time().seconds(), 0)
                .single()
                .unwrap_or_else(Local::now),
            subject: commit.summary().unwrap_or("(no subject)").to_string(),
        };

        let tree = commit.tree()?;
        let diff = if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            self.repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?
        } else {
            self.repo.diff_tree_to_tree(None, Some(&tree), None)?
        };

        let stats = diff.stats()?;

        Ok(CommitDetails {
            summary,
            body: commit.body().unwrap_or("").trim().to_string(),
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }

    pub fn branches(&self) -> Result<Vec<BranchInfo>> {
        let mut all = Vec::new();
        for kind in [BranchType::Local, BranchType::Remote] {
            for branch in self.repo.branches(Some(kind))? {
                let (branch, _) = branch?;
                let name = branch.name()?.unwrap_or("(invalid utf8)").to_string();
                let is_head = branch.is_head();
                let upstream = branch
                    .upstream()
                    .ok()
                    .and_then(|u| u.name().ok().flatten().map(ToString::to_string));

                all.push(BranchInfo {
                    name,
                    is_remote: kind == BranchType::Remote,
                    is_head,
                    upstream,
                });
            }
        }

        all.sort_by_key(|b| (b.is_remote, !b.is_head, b.name.clone()));
        Ok(all)
    }

    pub fn stash_count(&mut self) -> Result<usize> {
        let mut count = 0usize;
        self.repo.stash_foreach(|_, _, _| {
            count += 1;
            true
        })?;
        Ok(count)
    }

    pub fn graph_data(&self, limit: usize) -> Result<GraphData> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
        let _ = revwalk.push_glob("*");

        let mut nodes = Vec::new();
        let mut branches_map = HashMap::new();
        let mut branch_colors = Vec::new();
        let mut current_color = 0;

        for oid_result in revwalk.take(limit) {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;
            
            let mut branch_name = "unknown".to_string();
            if let Ok(branches) = self.repo.branches(Some(BranchType::Local)) {
                for b in branches {
                    if let Ok((b, _)) = b {
                        if b.get().target() == Some(oid) {
                            branch_name = b.name().ok().flatten().unwrap_or("").to_string();
                            break;
                        }
                    }
                }
            }

            let color_idx = *branches_map.entry(branch_name.clone()).or_insert_with(|| {
                let idx = current_color;
                branch_colors.push((branch_name.clone(), idx));
                current_color += 1;
                idx
            });

            nodes.push(GraphNode {
                id: oid.to_string(),
                short_id: format!("{oid:.8}"),
                parents: commit.parents().map(|p| p.id().to_string()).collect(),
                children: Vec::new(),
                branch: branch_name,
                subject: commit.summary().unwrap_or("").to_string(),
                lane: color_idx % 5,
                color_idx,
            });
        }

        let node_ids: HashMap<String, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id.clone(), i)).collect();
        for i in 0..nodes.len() {
            let child_id = nodes[i].id.clone();
            let parents = nodes[i].parents.clone();
            for p_id in parents {
                if let Some(&p_idx) = node_ids.get(&p_id) {
                    nodes[p_idx].children.push(child_id.clone());
                }
            }
        }

        Ok(GraphData {
            nodes,
            branches: branch_colors,
        })
    }

    pub fn graph_log(&self) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(["log", "--graph", "--oneline", "--all", "--decorate", "--color=always"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn collect_status(&self) -> Result<WorkingTreeStatus> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut summary = WorkingTreeStatus::default();

        for entry in statuses.iter() {
            let s = entry.status();

            if s.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED) {
                summary.staged += 1;
            }
            if s.intersects(Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED) {
                summary.modified += 1;
            }
            if s.contains(Status::WT_NEW) {
                summary.untracked += 1;
            }
            if s.contains(Status::CONFLICTED) {
                summary.conflicts += 1;
            }
        }

        Ok(summary)
    }

    fn operation_state(&self) -> Option<RepoOperation> {
        if self.repo.path().join("MERGE_HEAD").exists() {
            return Some(RepoOperation::Merge);
        }
        if self.repo.path().join("rebase-merge").exists()
            || self.repo.path().join("rebase-apply").exists()
        {
            return Some(RepoOperation::Rebase);
        }
        if self.repo.path().join("CHERRY_PICK_HEAD").exists() {
            return Some(RepoOperation::CherryPick);
        }
        None
    }

    fn current_upstream(&self) -> Option<String> {
        let head = self.repo.head().ok()?;
        let name = head.shorthand()?;
        let branch = self.repo.find_branch(name, BranchType::Local).ok()?;
        branch
            .upstream()
            .ok()
            .and_then(|b| b.name().ok().flatten().map(ToString::to_string))
    }
}
