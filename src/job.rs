pub struct Job {
    repo: String,
}

impl Job {
    pub fn new(repo: String) -> Self {
        Self { repo }
    }
    pub fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}
