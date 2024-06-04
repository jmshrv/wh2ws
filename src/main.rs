use std::{collections::HashMap, env, path::PathBuf};

use flume::Receiver;
use flume::Sender;
use rocket::futures::SinkExt;
use rocket::State;
use rocket_ws::Channel;
use rocket_ws::WebSocket;

#[macro_use]
extern crate rocket;

type WebhookMap = HashMap<String, (Sender<String>, Receiver<String>)>;

#[post("/webhook/<path..>", data = "<payload>")]
async fn webhook(path: PathBuf, payload: String, webhooks: &State<WebhookMap>) {
    let path = path.to_string_lossy().to_string();
    let webhook = webhooks.get(&path).expect("Invalid webhook path!");

    // Send payload to associated websocket stream
    webhook.0.send(payload).unwrap();
}

#[get("/websocket/<path..>?channel")]
fn websocket(path: PathBuf, ws: WebSocket, webhooks: &State<WebhookMap>) -> Channel<'static> {
    let path = path.to_string_lossy().to_string();
    let webhook = webhooks.get(&path).expect("Invalid webhook path!");

    let receiver = webhook.1.clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            while let Ok(message) = receiver.recv_async().await {
                stream.send(rocket_ws::Message::Text(message)).await?;
            }

            Ok(())
        })
    })
}

#[launch]
fn rocket() -> _ {
    let webhooks = env::var("WEBHOOKS")
        .expect("No WEBHOOKS environment variable set!")
        .split(',')
        .map(|path| (path.to_string(), flume::unbounded::<String>()))
        .collect::<HashMap<_, _>>();

    rocket::build()
        .mount("/", routes![webhook, websocket])
        .manage(webhooks)
}
