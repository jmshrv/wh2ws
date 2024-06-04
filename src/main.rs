use std::{collections::HashMap, env, path::PathBuf};

use rocket::futures::SinkExt;
use rocket::State;
use rocket_ws::Channel;
use rocket_ws::WebSocket;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;

#[macro_use]
extern crate rocket;

type WebhookMap = HashMap<String, (Sender<String>, Receiver<String>)>;

#[post("/webhook/<path..>", data = "<payload>")]
fn webhook(path: PathBuf, payload: String, webhooks: &State<WebhookMap>) {
    let path = path.to_string_lossy().to_string();
    let webhook = webhooks.get(&path).expect("Invalid webhook path!");

    // Send payload to associated websocket stream
    webhook.0.send(payload).unwrap();
}

#[get("/websocket/<path..>?channel")]
fn websocket(path: PathBuf, ws: WebSocket, webhooks: &State<WebhookMap>) -> Channel<'static> {
    let path = path.to_string_lossy().to_string();
    let webhook = webhooks.get(&path).expect("Invalid webhook path!");

    let mut receiver = webhook.0.subscribe();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            while let Ok(message) = receiver.recv().await {
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
        .map(|path| (path.to_string(), broadcast::channel::<String>(10000)))
        .collect::<HashMap<_, _>>();

    rocket::build()
        .mount("/", routes![webhook, websocket])
        .manage(webhooks)
}
