#![feature(proc_macro_hygiene, decl_macro)]

use std::env;
use std::path::PathBuf;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
use std::thread;

use rocket::{http::Status, post, routes, State};
use rocket_contrib::json::Json;

use log::error;

mod gitea;
use gitea::GiteaWebHook;

mod job;
use job::Job;

struct JobSender(Mutex<Sender<Job>>);

#[post("/", data = "<payload>")]
fn gitea_webhook(payload: Json<GiteaWebHook>, sender: State<JobSender>) -> Status {
    {
        let sender = sender.0.lock().unwrap();
        let repo = &payload.repository;
        sender.send(Job::new(repo.clone())).unwrap();
    }

    Status::Ok
}

fn repo_updater(rx: Receiver<Job>, homedir: PathBuf) {
    loop {
        let mut job = rx.recv().unwrap();

        if let Err(err) = job.run(&homedir) {
            error!("couldn't process job: {}", err);
        }
    }
}

fn main() {
    let (sender, receiver) = channel();

    let homedir = env::var("LOHR_HOME").unwrap_or_else(|_| "./".to_string());
    let homedir: PathBuf = homedir.into();
    let homedir = homedir.canonicalize().expect("LOHR_HOME isn't valid!");

    thread::spawn(move || {
        repo_updater(receiver, homedir);
    });

    rocket::ignite()
        .mount("/", routes![gitea_webhook])
        .manage(JobSender(Mutex::new(sender)))
        .launch();
}
