use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(crate) struct Repository {
    pub(crate) name: String,
    pub(crate) full_name: String,
    pub(crate) clone_url: String,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct GiteaWebHook {
    pub(crate) repository: Repository,
}
