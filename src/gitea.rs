use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct Repository {
    pub(crate) full_name: String,
    pub(crate) ssh_url: String,
}

#[derive(Deserialize)]
pub(crate) struct GiteaWebHook {
    pub(crate) repository: Repository,
}
