use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct Repository {
    pub(crate) name: String,
    pub(crate) full_name: String,
    pub(crate) clone_url: String,
}

#[derive(Deserialize)]
pub(crate) struct GiteaWebHook {
    pub(crate) repository: Repository,
}
