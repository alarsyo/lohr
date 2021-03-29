use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use anyhow::bail;

use log::info;

use crate::gitea::Repository;

pub(crate) struct Job {
    repo: Repository,
    local_path: Option<PathBuf>,
}

// TODO: implement git operations with git2-rs where possible

impl Job {
    const REMOTES: &'static [&'static str] = &["github", "gitlab"];

    pub(crate) fn new(repo: Repository) -> Self {
        Self {
            repo,
            local_path: None,
        }
    }

    fn repo_exists(&self) -> bool {
        self.local_path
            .as_ref()
            .map(|p| p.is_dir())
            .unwrap_or(false)
    }

    fn mirror_repo(&self) -> anyhow::Result<()> {
        info!("Cloning repo {}...", self.repo.full_name);

        let output = Command::new("git")
            .arg("clone")
            .arg("--mirror")
            .arg(&self.repo.clone_url)
            .arg(format!("{}", self.local_path.as_ref().unwrap().display()))
            .output()?;

        if !output.status.success() {
            let error = str::from_utf8(&output.stderr)?;
            let code = output
                .status
                .code()
                .unwrap_or_else(|| output.status.signal().unwrap());

            bail!(
                "couldn't mirror repo: exit code {}, stderr:\n{}",
                code,
                error
            );
        }

        // TODO: handle git LFS mirroring:
        // https://github.com/git-lfs/git-lfs/issues/2342#issuecomment-310323647

        Ok(())
    }

    fn update_repo(&self) -> anyhow::Result<()> {
        info!("Updating repo {}...", self.repo.full_name);

        let output = Command::new("git")
            .arg("-C")
            .arg(format!("{}", self.local_path.as_ref().unwrap().display()))
            .arg("remote")
            .arg("update")
            .arg("origin")
            // otherwise deleted tags and branches aren't updated on local copy
            .arg("--prune")
            .output()?;

        if !output.status.success() {
            let error = str::from_utf8(&output.stderr)?;
            let code = output
                .status
                .code()
                .unwrap_or_else(|| output.status.signal().unwrap());

            bail!(
                "couldn't update origin remote: exit code {}, stderr:\n{}",
                code,
                error
            );
        }

        Ok(())
    }

    fn update_mirrors(&self) -> anyhow::Result<()> {
        for remote in Self::REMOTES.iter() {
            info!("Updating mirror {}:{}...", remote, self.repo.full_name);

            let output = Command::new("git")
                .arg("-C")
                .arg(format!("{}", self.local_path.as_ref().unwrap().display()))
                .arg("push")
                .arg("--mirror")
                .arg(format!("git@{}.com:{}", remote, self.repo.full_name))
                .output()?;

            if !output.status.success() {
                let error = str::from_utf8(&output.stderr)?;
                let code = output
                    .status
                    .code()
                    .unwrap_or_else(|| output.status.signal().unwrap());

                bail!(
                    "couldn't update origin remote: exit code {}, stderr:\n{}",
                    code,
                    error
                );
            }
        }

        Ok(())
    }

    pub(crate) fn run(&mut self, homedir: &Path) -> anyhow::Result<()> {
        let local_path = homedir.join(&self.repo.full_name);
        println!("{}", local_path.display());
        assert!(local_path.is_absolute());
        self.local_path = Some(local_path);

        if !self.repo_exists() {
            self.mirror_repo()?;
        } else {
            self.update_repo()?;
        }

        self.update_mirrors()
    }
}
