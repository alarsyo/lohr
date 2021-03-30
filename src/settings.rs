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
    /// List of regexes, if a repository's name matches any of the, it is not mirrored by `lohr`
    /// even if it contains a `.lorh` file.
    #[serde(with = "serde_regex")]
    pub blacklist: Vec<regex::Regex>,
}
