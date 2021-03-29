#![feature(proc_macro_hygiene, decl_macro)]

use std::path::{Path, PathBuf};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
use std::thread;

use rocket::{fairing::AdHoc, http::Status, post, routes, State};
use rocket_contrib::json::Json;

use log::error;

mod gitea;
use gitea::GiteaWebHook;

mod job;
use job::Job;

struct HomeDir(PathBuf);
struct JobSender(Mutex<Sender<Job>>);

#[post("/", data = "<payload>")]
fn gitea_webhook(payload: Json<GiteaWebHook>, sender: State<JobSender>) -> Status {
    {
        let sender = sender.0.lock().unwrap();
        sender
            .send(Job::new(payload.repository.full_name.clone()))
            .unwrap();
    }

    Status::Ok
}

fn repo_updater(rx: Receiver<Job>) {
    loop {
        let job = rx.recv().unwrap();

        if let Err(err) = job.run() {
            error!("couldn't process job: {}", err);
        }
    }
}

fn main() {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        repo_updater(receiver);
    });

    rocket::ignite()
        .mount("/", routes![gitea_webhook])
        .manage(JobSender(Mutex::new(sender)))
        .attach(AdHoc::on_attach("Assets Config", |rocket| {
            let home_dir = rocket.config().get_str("home").unwrap();

            let home_dir = Path::new(home_dir).into();

            Ok(rocket.manage(HomeDir(home_dir)))
        }))
        .launch();
}
