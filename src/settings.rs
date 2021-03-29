use serde::Deserialize;

pub(crate) type RepoUrl = String; // FIXME: probably needs a better type than this

#[derive(Default, Deserialize)]
pub(crate) struct GlobalSettings {
    /// List of remote stems to use when no `.lohr` file is found
    #[serde(default)]
    pub default_remotes: Vec<RepoUrl>,
    /// List of remote stems to use for every repository
    #[serde(default)]
    pub additional_remotes: Vec<RepoUrl>,
}
