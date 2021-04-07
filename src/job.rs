use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use anyhow::bail;

use log::info;

use crate::gitea::Repository;
use crate::settings::{GlobalSettings, RepoUrl};

pub(crate) struct Job {
    repo: Repository,
    local_path: Option<PathBuf>,
}

// TODO: implement git operations with git2-rs where possible

impl Job {
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
            .arg(&self.repo.ssh_url)
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

    /// Can return Ok(None) if the .lohr file didn't exist, but no significant error occured
    fn read_remotes_from_lohr_file(&self) -> anyhow::Result<Option<Vec<RepoUrl>>> {
        // try to read .lohr file from bare repo (hence the git show sorcery)
        let output = Command::new("git")
            .arg("-C")
            .arg(format!("{}", self.local_path.as_ref().unwrap().display()))
            .arg("show")
            .arg("HEAD:.lohr")
            .output()?;

        if !output.status.success() {
            let error = str::from_utf8(&output.stderr)?;

            // this error case is okay, .lohr just doesn't exist
            if error.contains("does not exist in 'HEAD'") {
                return Ok(None);
            } else {
                let code = output
                    .status
                    .code()
                    .unwrap_or_else(|| output.status.signal().unwrap());

                bail!(
                    "couldn't read .lohr file from repo {}: exit code {}, stderr:\n{}",
                    self.repo.full_name,
                    code,
                    error
                );
            }
        }

        let output = String::from_utf8(output.stdout)?;

        Ok(Some(
            output
                .lines()
                .map(String::from)
                .filter(|s| !s.is_empty())
                .collect(),
        ))
    }

    fn get_remotes(&self, config: &GlobalSettings) -> anyhow::Result<Vec<RepoUrl>> {
        let local_path = self.local_path.as_ref().unwrap();

        let stem_to_repo = |stem: &RepoUrl| -> RepoUrl {
            let mut res = stem.clone();
            if !res.ends_with('/') {
                res.push('/');
            };
            res.push_str(local_path.file_name().unwrap().to_str().unwrap());
            res
        };

        // use either .lohr file or default remotes from config
        let mut remotes = match self.read_remotes_from_lohr_file()? {
            Some(remotes) if !remotes.is_empty() => remotes,
            _ => config.default_remotes.iter().map(stem_to_repo).collect(),
        };

        // additional remotes
        remotes.append(&mut config.additional_remotes.iter().map(stem_to_repo).collect());

        Ok(remotes)
    }

    fn update_mirrors(&self, config: &GlobalSettings) -> anyhow::Result<()> {
        for remote in &self.get_remotes(config)? {
            info!("Updating mirror {}...", remote);

            let output = Command::new("git")
                .arg("-C")
                .arg(format!("{}", self.local_path.as_ref().unwrap().display()))
                .arg("push")
                .arg("--mirror")
                .arg(remote)
                .output()?;

            if !output.status.success() {
                let error = str::from_utf8(&output.stderr)?;
                let code = output
                    .status
                    .code()
                    .unwrap_or_else(|| output.status.signal().unwrap());

                bail!(
                    "couldn't update remote {}: exit code {}, stderr:\n{}",
                    remote,
                    code,
                    error
                );
            }
        }

        Ok(())
    }

    pub(crate) fn run(&mut self, homedir: &Path, config: &GlobalSettings) -> anyhow::Result<()> {
        let local_path = homedir.join(&self.repo.full_name);
        assert!(local_path.is_absolute());
        self.local_path = Some(local_path);

        if !self.repo_exists() {
            self.mirror_repo()?;
        } else {
            self.update_repo()?;
        }

        self.update_mirrors(config)?;

        info!("Done processing job {}!", self.repo.full_name);

        Ok(())
    }
}
