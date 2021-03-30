#![feature(proc_macro_hygiene, decl_macro)]

use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
use std::thread;

use rocket::{http::Status, post, routes, State};

use log::error;

mod gitea;
use gitea::GiteaWebHook;

mod job;
use job::Job;

mod settings;
use settings::GlobalSettings;

mod signature;
use signature::SignedJson;

struct JobSender(Mutex<Sender<Job>>);
struct Secret(String);

#[post("/", data = "<payload>")]
fn gitea_webhook(payload: SignedJson<GiteaWebHook>, sender: State<JobSender>) -> Status {
    // TODO: validate Gitea signature

    {
        let sender = sender.0.lock().unwrap();
        let repo = &payload.repository;
        sender.send(Job::new(repo.clone())).unwrap();
    }

    Status::Ok
}

fn repo_updater(rx: Receiver<Job>, homedir: PathBuf, config: GlobalSettings) {
    loop {
        let mut job = rx.recv().unwrap();

        if let Err(err) = job.run(&homedir, &config) {
            error!("couldn't process job: {}", err);
        }
    }
}

fn parse_config(mut path: PathBuf) -> anyhow::Result<GlobalSettings> {
    path.push("lohr-config");
    path.set_extension("yaml");
    let config = if let Ok(file) = File::open(path.as_path()) {
        serde_yaml::from_reader(file)?
    } else {
        Default::default()
    };
    Ok(config)
}

fn main() -> anyhow::Result<()> {
    let (sender, receiver) = channel();

    let homedir = env::var("LOHR_HOME").unwrap_or_else(|_| "./".to_string());
    let homedir: PathBuf = homedir.into();
    let homedir = homedir.canonicalize().expect("LOHR_HOME isn't valid!");

    let secret = env::var("LOHR_SECRET")
        .expect("please provide a secret, otherwise anyone can send you a malicious webhook");

    let config = parse_config(homedir.clone())?;

    thread::spawn(move || {
        repo_updater(receiver, homedir, config);
    });

    rocket::ignite()
        .mount("/", routes![gitea_webhook])
        .manage(JobSender(Mutex::new(sender)))
        .manage(Secret(secret))
        .launch();

    Ok(())
}
