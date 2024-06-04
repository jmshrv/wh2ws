use std::{collections::HashMap, env, path::PathBuf};

use rocket::Data;
use tokio_stream::{self as stream, StreamExt};

#[macro_use]
extern crate rocket;

type WebhookMap = HashMap<String, stream::Pending<String>>;

#[post("/webhook/<path..>", data = "<payload>")]
fn webhook(path: PathBuf, payload: Data<'_>) {}

#[get("/websocket/<path..>")]
fn websocket(path: PathBuf) {}

#[launch]
fn rocket() -> _ {
    let webhooks = env::var("WEBHOOKS")
        .expect("No WEBHOOKS environment variable set!")
        .split(',')
        .map(|path| (path.to_string(), stream::pending::<String>()))
        .collect::<HashMap<_, _>>();

    rocket::build()
        .mount("/", routes![webhook])
        .manage(webhooks)
}
