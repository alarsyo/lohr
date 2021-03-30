#![feature(proc_macro_hygiene, decl_macro)]

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
use std::thread;

use anyhow::Context;
use clap::{App, Arg};
use log::{error, info};
use rocket::{http::Status, post, routes, State};

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
fn gitea_webhook(
    payload: SignedJson<GiteaWebHook>,
    sender: State<JobSender>,
    config: State<GlobalSettings>,
) -> Status {
    if config
        .blacklist
        .iter()
        .any(|re| re.is_match(&payload.repository.full_name))
    {
        info!(
            "Ignoring webhook for repo {} which is blacklisted",
            payload.repository.full_name
        );
        return Status::Ok;
    }

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

fn parse_config(home: &Path, flags: &clap::ArgMatches) -> anyhow::Result<GlobalSettings> {
    // prioritize CLI flag, then env var
    let config_path = flags.value_of("config").map(PathBuf::from);
    let config_path = config_path.or_else(|| env::var("LOHR_CONFIG").map(PathBuf::from).ok());

    let file = match config_path {
        Some(config_path) => File::open(&config_path).with_context(|| {
            format!(
                "could not open provided configuration file at {}",
                config_path.display()
            )
        })?,
        None => {
            // check if file exists in lohr home
            let config_path = home.join("lohr-config.yaml");
            if !config_path.is_file() {
                return Ok(Default::default());
            }

            File::open(config_path).context("failed to open configuration file in LOHR_HOME")?
        }
    };

    serde_yaml::from_reader(file).context("could not parse configuration file")
}

fn main() -> anyhow::Result<()> {
    let matches = App::new("lohr")
        .version("0.3.1")
        .about("Git mirroring daemon")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Use a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let (sender, receiver) = channel();

    let homedir = env::var("LOHR_HOME").unwrap_or_else(|_| "./".to_string());
    let homedir: PathBuf = homedir.into();
    let homedir = homedir.canonicalize().expect("LOHR_HOME isn't valid!");

    let secret = env::var("LOHR_SECRET")
        .expect("please provide a secret, otherwise anyone can send you a malicious webhook");

    let config = parse_config(&homedir, &matches)?;
    let config_state = config.clone();

    thread::spawn(move || {
        repo_updater(receiver, homedir, config);
    });

    rocket::ignite()
        .mount("/", routes![gitea_webhook])
        .manage(JobSender(Mutex::new(sender)))
        .manage(Secret(secret))
        .manage(config_state)
        .launch();

    Ok(())
}
